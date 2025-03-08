use rocket::tokio::net::TcpStream;
use rocket::tokio::io::AsyncWriteExt;
use std::error::Error;

pub async fn connect_to_backend() -> Result<(), Box<dyn Error>> {
    println!("I connected to the backend!");
    // Connect to a peer
    let mut stream = TcpStream::connect("127.0.0.1:9999").await?;
    // Write some data.
    stream.write_all(b"hello world!").await?;

    Ok(())
}
