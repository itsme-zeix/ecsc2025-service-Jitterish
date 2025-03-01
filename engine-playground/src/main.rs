mod sandbox;

use crate::sandbox::sandbox::run_sandboxed;
use std::error::Error;
use std::time::Duration;
use unshare::Stdio;

async fn run_simple() {
    let cmd = "ls /";
    let mut child = run_sandboxed(
        unshare::Command::new("/usr/bin/sh").arg("-c").arg(cmd).stdout(Stdio::null()),
        ".",
        Duration::from_millis(500),
    )
    .await.unwrap();
    eprintln!("{} the command exited with: {}", child.pid(), child.wait().unwrap());
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

    let mut i = 0;
    loop {
        tokio::join!(
            run_simple(),
            run_simple(),
            run_simple(),
            run_simple(),
            run_simple(),
            run_simple(),
            run_simple(),
            run_simple(),
            run_simple(),
            run_simple(),
            run_simple(),
            run_simple(),
            run_simple(),
            run_simple(),
            run_simple(),
            run_simple(),
            run_simple(),
            run_simple(),
            run_simple(),
            run_simple(),
            run_simple(),
            run_simple(),
            run_simple(),
            run_simple(),
            run_simple(),
            run_simple(),
            run_simple(),
            run_simple(),
            run_simple(),
            run_simple(),
            run_simple(),
            run_simple(),
            run_simple(),
            run_simple(),
            run_simple(),
            run_simple(),
            run_simple(),
            run_simple(),
            run_simple(),
            run_simple(),
            run_simple(),
            run_simple(),
            run_simple(),
            run_simple(),
            run_simple(),
            run_simple(),
            run_simple(),
            run_simple(),
        );
        i += 1;
        println!("completed {}", i);
        // tokio::time::sleep(Duration::from_millis(200)).await;
        if i > 2000 {
            break;
        }
    }

    Ok(())
}

// docker run --security-opt apparmor=unconfined --cap-add SYS_ADMIN --rm --name tmp -it -v .:/app -w /app -e RUST_BACKTRACE=1 debian:trixie target/debug/engine-playground
