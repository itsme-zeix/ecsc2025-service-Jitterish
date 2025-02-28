use crate::sandbox::async_unshare::wait_async;
use nix::mount::{mount, umount2, MntFlags, MsFlags};
use nix::sys::resource::{setrlimit, Resource};
use nix::unistd::{chdir, chroot, getgid, getuid, pivot_root, Uid};
use std::io::{ErrorKind, Read};
use std::path::Path;
use std::time::Duration;
use std::{fs, io, path};
use tokio::time::timeout;
use unshare::{ExitStatus, GidMap, Namespace, UidMap};

fn sandbox_mount(uid: Uid, data_folder: &str) -> Result<(), io::Error> {
    let destdir = format!("/run/user/{}/sandbox_root", uid);
    fs::create_dir_all(destdir.as_str()).expect("failed to create sandbox root dir");

    // keep all mount changes private
    mount::<str, str, str, str>(
        Some("/"),
        "/",
        None,
        MsFlags::MS_PRIVATE | MsFlags::MS_REC,
        None,
    )
    .map_err(|e| io::Error::new(ErrorKind::Other, format!("mount #1 {}", e).as_str()))?;

    // root filesystem
    mount::<str, str, str, str>(
        None,
        destdir.as_str(),
        Some("tmpfs"),
        MsFlags::empty(),
        None,
    )
    .map_err(|e| io::Error::new(ErrorKind::Other, format!("mount #2 {}", e).as_str()))?;

    // read-only access limited to some folders
    for d in ["/bin", "/etc", "/lib", "/lib64", "/usr"] {
        let mountpoint = format!("{}{}", destdir.as_str(), d);
        fs::create_dir_all(mountpoint.as_str()).expect("failed to create dir in sandbox");
        mount::<str, str, str, str>(
            Some(d),
            mountpoint.as_str(),
            Some(""),
            MsFlags::MS_PRIVATE | MsFlags::MS_REC | MsFlags::MS_BIND,
            Some(""),
        )?;
    }
    // access limited to some devices
    for f in ["/dev/urandom"] {
        let mountpoint = format!("{}{}", destdir.as_str(), f);
        let path = Path::new(mountpoint.as_str());
        fs::create_dir_all(path.parent().unwrap()).expect("failed to create dir in sandbox");
        fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(path)
            .unwrap();
        mount::<str, str, str, str>(
            Some(f),
            mountpoint.as_str(),
            None,
            MsFlags::MS_PRIVATE | MsFlags::MS_REC | MsFlags::MS_BIND,
            None,
        )?;
    }

    // mount new /proc (not in docker)
    let mountpoint = format!("{}{}", destdir.as_str(), "/proc");
    fs::create_dir_all(mountpoint.as_str()).expect("failed to create dir in sandbox");
    let _ = mount::<str, str, str, str>(
        None,
        mountpoint.as_str(),
        Some("proc"),
        MsFlags::empty(),
        None,
    );

    // mount data
    let mountpoint = format!("{}{}", destdir.as_str(), "/data");
    fs::create_dir_all(mountpoint.as_str()).expect("failed to create dir in sandbox");
    mount::<str, str, str, str>(
        Some(data_folder),
        mountpoint.as_str(),
        None,
        MsFlags::MS_PRIVATE | MsFlags::MS_REC | MsFlags::MS_BIND | MsFlags::MS_RDONLY,
        None,
    )?;

    // chroot jail (try pivot_root first)
    pivot_root(destdir.as_str(), destdir.as_str())
        .and_then(|_| umount2("/", MntFlags::MNT_DETACH))
        .or_else(|_| {
            // fallback for docker containers with seccomp enabled
            chdir(destdir.as_str())?;
            mount::<str, str, str, str>(Some("."), "/", None, MsFlags::MS_MOVE, None)?;
            chroot(".")
        })?;
    chdir("/data")?;

    // make things actually read-only
    for d in ["/", "/bin", "/etc", "/lib", "/lib64", "/usr", "/data"] {
        mount::<str, str, str, str>(
            Some(d),
            d,
            None,
            MsFlags::MS_REMOUNT | MsFlags::MS_BIND | MsFlags::MS_RDONLY,
            None,
        )?;
    }

    Ok(())
}

fn sandbox_rlimit() -> nix::Result<()> {
    setrlimit(Resource::RLIMIT_NOFILE, 10, 10)?;
    setrlimit(Resource::RLIMIT_CPU, 1, 1)?;
    setrlimit(Resource::RLIMIT_NPROC, 3, 3)?;
    setrlimit(Resource::RLIMIT_AS, 64 * 1024 * 1024, 64 * 1024 * 1024)?;
    setrlimit(Resource::RLIMIT_RSS, 64 * 1024 * 1024, 64 * 1024 * 1024)?;
    setrlimit(Resource::RLIMIT_NICE, 0, 0)?;
    Ok(())
}

pub async fn run_sandboxed(
    cmd: &mut unshare::Command,
    folder: &str,
    runtime: Duration,
) -> Result<unshare::Child, Box<dyn std::error::Error>> {
    let my_uid = getuid();
    let my_gid = getgid();

    // Namespaces limit access to system (no network etc)
    cmd.unshare(&[
        Namespace::User,
        Namespace::Mount,
        Namespace::Ipc,
        Namespace::Pid,
        Namespace::Net,
        Namespace::Uts,
        Namespace::Cgroup,
    ]);
    cmd.set_id_maps(
        [UidMap {
            inside_uid: 0,
            outside_uid: my_uid.into(),
            count: 1,
        }]
        .into(),
        [GidMap {
            inside_gid: 99999,
            outside_gid: my_gid.into(),
            count: 1,
        }]
        .into(),
    );
    cmd.uid(0);
    cmd.gid(99999);

    // Filesystem and resource limits
    let folder = path::absolute(folder)?;
    let pre_exec = move || {
        sandbox_mount(my_uid, folder.to_str().unwrap()).map_err(|e| {
            eprintln!("sandbox_mount error: {}", e);
            e
        })?;
        sandbox_rlimit()?;

        Ok(())
    };
    unsafe {
        cmd.pre_exec(pre_exec);
    }

    // TODO check file descriptors

    let mut child = cmd.spawn().expect("failed to spawn");
    let wait_result = timeout(runtime, wait_async(&mut child)).await;

    let termination_result = match wait_result {
        Ok(x) => x,
        Err(_) => {
            child.kill()?;
            wait_async(&mut child).await
        }
    };

    Ok(child)
}

#[cfg(test)]
#[path = "./sandbox_test.rs"]
mod test;
