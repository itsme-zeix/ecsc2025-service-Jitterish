use interface::database::{CodeRef, Collection, Command};
use rocket::tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use rocket::tokio::net::TcpStream;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use std::io::ErrorKind;

pub struct DbConnection {
    stream: BufReader<TcpStream>,
}

pub fn one<T>(vec: &Vec<T>) -> std::io::Result<&T> {
    vec.last()
        .ok_or_else(|| std::io::Error::new(ErrorKind::InvalidData, "no result"))
}

impl DbConnection {
    pub async fn open(database: &str) -> std::io::Result<DbConnection> {
        let mut conn = DbConnection {
            stream: BufReader::new(TcpStream::connect("127.0.0.1:9999").await?),
        };
        conn.send(&Command::Select { database }).await?;
        conn.recv::<Value>().await?;

        Ok(conn)
    }

    async fn send<T: Serialize>(self: &mut DbConnection, cmd: &T) -> std::io::Result<()> {
        let data = serde_json::to_vec(cmd)?;
        self.stream.write_all(data.as_slice()).await?;
        self.stream.write_all(b"\n").await
    }

    async fn recv<T: DeserializeOwned>(self: &mut DbConnection) -> std::io::Result<Vec<T>> {
        let mut result: Vec<T> = Vec::new();
        loop {
            let mut line = String::new();
            if self.stream.read_line(&mut line).await? <= 0 {
                return Err(std::io::Error::new(ErrorKind::BrokenPipe, "EOF"));
            }

            match serde_json::from_str(&line) {
                Ok(value) => {
                    serde_json::from_value(value).ok().map(|t| result.push(t));
                }
                Err(_) => {
                    return if line == "OK\n" {
                        Ok(result)
                    } else {
                        Err(std::io::Error::new(ErrorKind::InvalidData, line.trim()))
                    }
                }
            }
        }
    }

    pub async fn list_collections(self: &mut DbConnection) -> std::io::Result<Vec<Collection>> {
        self.send(&Command::List).await?;
        self.recv::<Collection>().await
    }

    pub async fn append<T: Serialize>(self: &mut DbConnection, collection: &str, value: T) -> std::io::Result<()> {
        let data = serde_json::to_value(&value)?;
        self.send(&Command::Append { collection, data }).await?;
        self.recv::<Value>().await.map(|_| ())
    }

    pub async fn prepare(self: &mut DbConnection, code: &str) -> std::io::Result<CodeRef> {
        self.send(&Command::Prepare { code: code.to_owned() }).await?;
        let result = self.recv::<CodeRef>().await?;
        Ok(one(&result)?.clone())
    }

    /// Return all println() results + the return value of a function/query execution
    pub async fn execute<TS: Serialize, TD: DeserializeOwned>(
        self: &mut DbConnection,
        code_ref: &CodeRef,
        query: &str,
        param: TS,
    ) -> std::io::Result<Vec<TD>> {
        let param = serde_json::to_value(param)?;
        self.send(&Command::Execute {
            code_ref: code_ref.code_ref.as_str(),
            query,
            param,
        })
        .await?;
        self.recv::<TD>().await
    }

    /// Return the return value of a function/query execution only (ignoring println)
    pub async fn execute_simple<TS: Serialize, TD: DeserializeOwned + Clone>(
        self: &mut DbConnection,
        code_ref: &CodeRef,
        query: &str,
        param: TS,
    ) -> std::io::Result<Vec<TD>> {
        let result: Vec<Vec<TD>> = self.execute(code_ref, query, param).await?;
        Ok(one(&result)?.clone())
    }
}
