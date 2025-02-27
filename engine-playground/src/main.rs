mod async_unshare;

use crate::async_unshare::wait_async;
use std::future::Future;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::sleep;
use unshare::Namespace;

async fn test1() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::new("sh");
    cmd.arg("-c").arg("id ; pwd ; ls -l");

    cmd.uid(1);
    unsafe {
        cmd.pre_exec(|| {
            // fs::chroot("/")?;
            std::env::set_current_dir("/")?;
            println!("pre-exec");
            Ok(())
        });
    }

    let mut child = cmd.spawn().expect("failed to spawn");

    // Await until the command completes
    let status = child.wait().await?;
    println!("the command exited with: {}", status);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = unshare::Command::new("/usr/bin/sh");
    cmd.arg("-c")
        .arg("id ; pwd ; ls -l ; ip link ; ls -l /proc/self/fd ; sleep 1 ; exit 1");

    cmd.unshare(&[
        Namespace::User,
        Namespace::Pid,
        Namespace::Ipc,
        Namespace::Net,
        Namespace::Mount,
    ]);
    // cmd.set_id_maps({1: 1}, {1: 1});
    // TODO chroot
    // TODO user
    // TODO network looks good
    // TODO check file descriptors
    // TODO ulimit

    let mut child = cmd.spawn().expect("failed to spawn");

    // sleep(Duration::from_millis(5000)).await;

    let status = wait_async(&mut child).await?;
    println!("the command exited with: {}", status);

    /*
    let status2 = wait_async(&mut child).await?;
    println!("the command exited with: {}", status2);
    // */

    Ok(())
}
