mod sandbox;

use crate::sandbox::sandbox::run_sandboxed;
use std::error::Error;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // let cmd = "id ; pwd ; ls -l ; ip link ; ls -l /proc/self/fd ; ulimit -a ; sleep 5 ; touch test.txt ; exit 2";
    let cmd = "id ; pwd ; ls -l . / ; mount ; ls -l /proc/self/fd ; touch test.txt ; ps -aux ; ip link ; getpcaps $$";
    let mut child = run_sandboxed(
        unshare::Command::new("/usr/bin/sh").arg("-c").arg(cmd),
        "./src/",
        Duration::from_millis(500),
    )
    .await?;
    println!("the command exited with: {}", child.wait()?);

    Ok(())
}

// docker run --security-opt apparmor=unconfined --cap-add SYS_ADMIN --rm --name tmp -it -v .:/app -w /app -e RUST_BACKTRACE=1 debian:trixie target/debug/engine-playground
