mod async_unshare;

use crate::async_unshare::wait_async;
use nix::sys::resource::{setrlimit, Resource};
use std::error::Error;
use std::time::Duration;
use tokio::time::timeout;
use unshare::{ExitStatus, Namespace};

async fn run_sandboxed(
    s: &str,
    folder: &str,
    runtime: Duration,
) -> Result<i32, Box<dyn std::error::Error>> {
    let mut cmd = unshare::Command::new("/usr/bin/sh");
    cmd.arg("-c").arg(s);

    // Namespaces limit access to system (no network etc)
    cmd.unshare(&[
        Namespace::User,
        Namespace::Mount,
        Namespace::Ipc,
        Namespace::Pid,
        Namespace::Net,
    ]);
    // Resource limits
    unsafe {
        cmd.pre_exec(|| {
            setrlimit(Resource::RLIMIT_NOFILE, 10, 10)?;
            setrlimit(Resource::RLIMIT_CPU, 1, 1)?;
            setrlimit(Resource::RLIMIT_NPROC, 3, 3)?;
            setrlimit(Resource::RLIMIT_AS, 64 * 1024 * 1024, 64 * 1024 * 1024)?;
            setrlimit(Resource::RLIMIT_RSS, 64 * 1024 * 1024, 64 * 1024 * 1024)?;
            setrlimit(Resource::RLIMIT_NICE, 0, 0)?;
            Ok(())
        });
    }

    cmd.current_dir(folder);
    // cmd.pivot_root(folder, folder.to_owned() + "root/", false);
    // cmd.chroot_dir(folder);

    // cmd.set_id_maps({1: 1}, {1: 1});
    // TODO chroot
    // TODO user
    // TODO network looks good
    // TODO check file descriptors
    // TODO test everything in a docker container

    let mut child = cmd.spawn().expect("failed to spawn");
    let wait_result = timeout(runtime, wait_async(&mut child)).await;

    let termination_result = match wait_result {
        Ok(x) => x,
        Err(_) => {
            child.kill()?;
            wait_async(&mut child).await
        }
    };

    match termination_result? {
        ExitStatus::Exited(rc) => Ok(rc.into()),
        ExitStatus::Signaled(sig, _) => Ok(128 + sig as i32),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cmd = "id ; pwd ; ls -l ; ip link ; ls -l /proc/self/fd ; ulimit -a ; sleep 5 ; touch test.txt ; exit 2";
    let status = run_sandboxed(
        cmd,
        "/home/markus/Projekte/saarsec/Jitterish/engine-playground/src/",
        Duration::from_millis(500),
    )
    .await?;
    println!("the command exited with: {}", status);

    Ok(())
}
