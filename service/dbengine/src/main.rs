mod db;
mod sandbox;

use crate::db::execute_query;
use interface::database::{CodeRef, Collection, Command};
use serde::Serialize;
use std::error::Error;
use std::io::ErrorKind;
use std::os::fd::AsFd;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::{select, signal};
use tokio_util::sync::CancellationToken;
use crate::sandbox::sandbox::compiler_path;

async fn write_json<T: Serialize>(stream: &mut BufReader<TcpStream>, data: &T) -> std::io::Result<()> {
    let data = serde_json::to_vec(data)?;
    stream.write_all(data.as_slice()).await?;
    stream.write_all(b"\n").await
}

async fn connection(socket: TcpStream) -> std::io::Result<()> {
    let mut stream = BufReader::new(socket);
    let mut buffer = String::new();
    let mut current_database = "none".to_string();

    loop {
        if stream.read_line(&mut buffer).await? <= 0 {
            return Ok(());
        }

        match serde_json::from_str::<Command>(buffer.as_str()) {
            Ok(cmd) => {
                match command(&mut stream, &mut current_database, cmd).await {
                    Ok(()) => stream.write_all("OK\n".as_bytes()).await?,
                    Err(e) => stream.write_all(format!("Failed command: {}\n", e).as_bytes()).await?,
                }
                buffer.clear();
            }
            Err(e) => {
                if !e.is_eof() {
                    stream.write_all(format!("Invalid command: {}\n", e).as_bytes()).await?;
                    return Err(std::io::Error::new(ErrorKind::Other, e));
                }
            }
        }
    }
}

async fn command(
    stream: &mut BufReader<TcpStream>,
    current_database: &mut String,
    command: Command<'_>,
) -> std::io::Result<()> {
    match command {
        Command::Select { database } => {
            current_database.clear();
            current_database.push_str(database);
        }

        Command::List => {
            for collection in db::list_collections(current_database)? {
                let result = &Collection { name: collection };
                write_json(stream, result).await?;
            }
        }

        Command::Append { collection, data } => {
            db::append(current_database.as_str(), collection, &data)?;
        }

        Command::Prepare { code } => {
            let code_ref = db::prepare_query(&code).await?;
            let result = &CodeRef { code_ref };
            write_json(stream, result).await?;
        }

        Command::Execute { code_ref, query, param, privileged } => {
            execute_query(current_database, code_ref, query, &param, privileged, stream.get_ref().as_fd()).await?;
        }
    }
    Ok(())
}

async fn server(cancel: CancellationToken) -> std::io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:9999").await?;

    loop {
        select! {
            Ok(( socket, _addr)) = listener.accept() => {
                tokio::spawn(connection(socket));
            }
            _ = cancel.cancelled() => {
                return Ok(());
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Compiler path: {}", compiler_path.to_str().unwrap());

    let token = CancellationToken::new();
    let handle = tokio::spawn(server(token.clone()));
    signal::ctrl_c().await?;
    eprintln!("Shutting down...");
    token.cancel();
    handle.await?.expect("Error waiting for termination signal");
    eprintln!("Server terminated.");
    Ok(())
}

// docker run --security-opt apparmor=unconfined --cap-add SYS_ADMIN --rm --name tmp -it -v .:/app -w /app -e RUST_BACKTRACE=1 debian:trixie target/debug/engine-playground
