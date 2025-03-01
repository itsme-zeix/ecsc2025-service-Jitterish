use crate::sandbox::sandbox::{compiler_path, run_sandboxed};
use std::error::Error;
use std::path::Path;
use std::time::Duration;
use unshare::{Command, ExitStatus};

fn check_exit_status(status: ExitStatus) -> Result<(), Box<dyn Error>> {
    match status {
        ExitStatus::Exited(0) => Ok(()),
        ExitStatus::Exited(rc) => Err(format!("error code {}", rc).into()),
        ExitStatus::Signaled(sig, _) => Err(format!("terminated by signal {}", sig).into()),
    }
}

pub async fn compile<P: AsRef<Path>>(code: &str, output: P) -> Result<(), Box<dyn Error>> {
    let mut cmd = Command::new("/app/jit_playground");
    cmd.arg("-").arg("-");

    // TODO stdin
    // TODO stdout

    let mut child = run_sandboxed::<&str>(&mut cmd, None, Duration::from_millis(1000)).await?;
    check_exit_status(child.wait()?)
}

pub async fn run<P: AsRef<Path>>(
    binary: P,
    directory: P,
    query: &str,
    param: &str,
) -> Result<(), Box<dyn Error>> {
    // TODO output file descriptor / connection etc

    let mut cmd = Command::new(binary.as_ref().as_os_str());
    cmd.arg(query).arg(param);

    // TODO stdin
    // TODO stdout

    let mut child = run_sandboxed(&mut cmd, Some(directory), Duration::from_millis(250)).await?;
    check_exit_status(child.wait()?)
}
