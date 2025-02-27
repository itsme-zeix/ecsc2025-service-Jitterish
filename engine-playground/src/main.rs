mod async_unshare;

use crate::async_unshare::wait_async;
use nix::sys::resource::{setrlimit, Resource};
use std::future::Future;
use unshare::{ExitStatus, Namespace};

async fn run_sandboxed(s: &str, folder: &str) -> Result<i32, Box<dyn std::error::Error>> {
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
    // cmd.set_id_maps({1: 1}, {1: 1});
    // TODO chroot
    // TODO user
    // TODO network looks good
    // TODO check file descriptors

    let mut child = cmd.spawn().expect("failed to spawn");
    match wait_async(&mut child).await? {
        ExitStatus::Exited(rc) => Ok(rc.into()),
        ExitStatus::Signaled(sig, _) => Ok(128 + sig as i32),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cmd = "id ; pwd ; ls -l ; ip link ; ls -l /proc/self/fd ; ulimit -a ; exit 2";
    let status = run_sandboxed(cmd, "./src/").await?;
    println!("the command exited with: {}", status);

    Ok(())
}
