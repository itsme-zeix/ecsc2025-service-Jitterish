use crate::sandbox::sandbox::run_sandboxed;
use nix::sys::memfd::{memfd_create, MemFdCreateFlag};
use nix::unistd::{lseek, write, Whence};
use std::error::Error;
use std::ffi::CString;
use std::fs::OpenOptions;
use std::os::fd::{AsFd, AsRawFd, BorrowedFd};
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use std::time::Duration;
use unshare::{Command, ExitStatus, Stdio};

fn check_exit_status(status: ExitStatus) -> Result<(), Box<dyn Error>> {
    match status {
        ExitStatus::Exited(0) => Ok(()),
        ExitStatus::Exited(rc) => Err(format!("error code {}", rc).into()),
        ExitStatus::Signaled(sig, _) => Err(format!("terminated by signal {}", sig).into()),
    }
}

pub async fn compile<P: AsRef<Path>>(code: &str, output: P) -> Result<(), Box<dyn Error>> {
    let mut cmd = Command::new("/app/compiler/jit_playground");
    cmd.arg("-").arg("-");

    // program code to stdin
    let memfd = memfd_create(
        CString::new(b"code").unwrap().as_c_str(),
        MemFdCreateFlag::MFD_CLOEXEC,
    )?;
    write(memfd.as_fd(), &code.as_bytes())?;
    lseek(memfd.as_raw_fd(), 0, Whence::SeekSet)?;
    cmd.stdin(Stdio::from_file(memfd));

    // output file to stdout
    cmd.stdout(Stdio::from_file(OpenOptions::new().mode(0o755).create(true).write(true).open(output)?));

    let mut child = run_sandboxed::<&str>(cmd, None, None, Duration::from_millis(1000), 384).await?;
    check_exit_status(child.wait()?)
    // TODO delete file on error!
}

pub async fn run<P: AsRef<Path>>(
    binary: P,
    directory: P,
    query: &str,
    param: &str,
    output: BorrowedFd<'_>,
) -> Result<(), Box<dyn Error>> {
    // TODO output file descriptor / connection etc

    let mut cmd = Command::new("/app/executable");
    cmd.arg(query).arg(param);
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::dup_file(&output)?);

    let mut child = run_sandboxed(
        cmd,
        Some(binary),
        Some(directory),
        Duration::from_millis(250),
        32,
    )
    .await?;
    check_exit_status(child.wait()?)
}
