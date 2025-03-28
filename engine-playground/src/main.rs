mod db;
mod sandbox;

use std::error::Error;

use interface::database::{Collection, Command};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::{select, signal};
use tokio_util::sync::CancellationToken;

async fn write_json<T: serde::Serialize>(stream: &mut BufReader<TcpStream>, data: &T) -> std::io::Result<()> {
    let data = serde_json::to_vec(data)?;
    stream.write_all(data.as_slice()).await?;
    stream.write_all(b"\n").await
}

async fn connection(socket: TcpStream) -> std::io::Result<()> {
    let mut stream = BufReader::new(socket);

    let mut current_database = "none".to_string();

    loop {
        let mut line = String::new();
        if stream.read_line(&mut line).await? <= 0 {
            return Ok(());
        }

        let cmd: serde_json::Result<Command> = serde_json::from_str(line.as_str());
        match command(&mut stream, &mut current_database, cmd?).await {
            Ok(()) => {
                stream.write_all("OK\n".as_bytes()).await?;
            }
            Err(e) => {
                stream.write_all(format!("ERROR {}\n", e).as_bytes()).await?;
            }
        }
    }
}

async fn command(stream: &mut BufReader<TcpStream>, current_database: &mut String, command: Command<'_>) -> std::io::Result<()> {
    match command {
        Command::Select { database } => {
            current_database.clear();
            current_database.push_str(database);
        }

        Command::List => {
            for collection in db::list_collections(current_database)? {
                let result = &Collection {
                    name: collection.as_str(),
                };
                write_json(stream, result).await?;
            }
        }

        Command::Append { collection, data } => {
            db::append(current_database.as_str(), collection, &data)?;
        }

        Command::Prepare { .. } => {
            // TODO
        }

        Command::Execute { .. } => {
            // TODO
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
    let token = CancellationToken::new();
    let handle = tokio::spawn(server(token.clone()));
    signal::ctrl_c().await?;
    eprintln!("Shutting down...");
    token.cancel();
    handle.await?.expect("TODO");
    eprintln!("Server terminated.");

    // compiler::compile("func jit_entrypoint() { return \"Hello World\"; }", "./data/output.so").await?;
    // compiler::run("./data/output.so", "./data", "jit_entrypoint", "null").await?;

    Ok(())
}

// docker run --security-opt apparmor=unconfined --cap-add SYS_ADMIN --rm --name tmp -it -v .:/app -w /app -e RUST_BACKTRACE=1 debian:trixie target/debug/engine-playground
