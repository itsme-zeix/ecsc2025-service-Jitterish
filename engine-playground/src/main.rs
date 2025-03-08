mod sandbox;

use crate::sandbox::compiler;
use crate::sandbox::sandbox::run_sandboxed;
use std::error::Error;
use std::time::Duration;
use std::str;
use unshare::Stdio;

use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncWriteExt,AsyncReadExt};
use tokio::{select, signal};
use tokio::signal::unix::SignalKind;
use tokio_util::sync::CancellationToken;

async fn frontend_connection(mut socket: TcpStream) -> std::io::Result<()>{
    println!("Writing to socket");
    let mut buf = vec![0; 1024];
    let n = socket
        .read(&mut buf)
        .await?;
    let s = match str::from_utf8(buf.as_slice()) {
        Ok(v) => v,
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    };
    println!("result: {}", s);
    socket.write_all("Hello\n".as_bytes()).await?;
    //println!("Write result: {:?}", res);
    socket.shutdown().await?;
    Ok(())
}

async fn frontend_listener(cancel: CancellationToken) -> std::io::Result<()>{
    let listener = TcpListener::bind("0.0.0.0:9999").await?;

    loop {
        select! {
            Ok((mut socket, _addr)) = listener.accept() => {
                tokio::spawn(frontend_connection(socket));
            }
            _ = cancel.cancelled() => {
                return Ok(());
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // let cmd = "id ; pwd ; ls -l ; ip link ; ls -l /proc/self/fd ; ulimit -a ; sleep 5 ; touch test.txt ; exit 2";
    //let cmd = "id ; pwd ; ls -l . / ; mount ; ls -l /proc/self/fd ; touch test.txt ; ps -aux ; ip link ; getpcaps $$";
    /*let cmd = "id";
    let mut child = run_sandboxed(
        unshare::Command::new("/usr/bin/sh").arg("-c").arg(cmd),
        "./src/",
        Duration::from_millis(500000),
    )
    .await?;
    println!("the command exited with: {}", child.wait()?);
    // */

    let token = CancellationToken::new();
    let handle = tokio::spawn(frontend_listener(token.clone()));
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
