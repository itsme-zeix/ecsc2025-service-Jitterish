use crate::sandbox::async_unshare::wait_async;
use lazy_static::lazy_static;
use nix::libc;
use nix::libc::{MS_BIND, MS_PRIVATE, MS_RDONLY, MS_REC, MS_REMOUNT};
use nix::mount::{mount, MsFlags};
use nix::sys::resource::{setrlimit, Resource};
use nix::unistd::{getgid, getuid, Uid};
use std::ffi::{c_char, c_int, c_void};
use std::io::Read;
use std::time::Duration;
use std::{fs, io, path};
use tokio::time::timeout;
use unshare::{ExitStatus, GidMap, Namespace, UidMap};

extern "C" {
    pub fn pivot_root(new_root: *const c_char, put_old: *const c_char) -> c_int;
}

unsafe fn mount_dir_p1(dst: *const u8, src: *const u8) {
    libc::mkdir(dst as *const c_char, 0o777);
    if libc::mount(
        src as *const c_char,
        dst as *const c_char,
        0 as *const c_char,
        MS_PRIVATE | MS_REC | MS_BIND,
        0 as *const c_void,
    ) < 0
    {
        panic!("failed to mount directory");
    }
}

unsafe fn remount_dir(dir: *const u8) {
    if libc::mount(
        dir as *const c_char,
        dir as *const c_char,
        0 as *const c_char,
        MS_REMOUNT | MS_BIND | MS_RDONLY,
        0 as *const c_void,
    ) < 0
    {
        panic!("failed to remount directory");
    }
}

fn mount_device(dst: &str, src: &str) {
    fs::create_dir_all("/dev/shm/sandbox_root/dev").expect("failed to create dir in sandbox");
    fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(dst)
        .unwrap();
    mount::<str, str, str, str>(
        Some(src),
        dst,
        None,
        MsFlags::MS_PRIVATE | MsFlags::MS_REC | MsFlags::MS_BIND,
        None,
    )
        .expect("cannot mount device");
}

fn sandbox_mount(uid: Uid, data_folder: *const u8) -> nix::Result<()> {
    unsafe {
        // keep all mount changes private
        libc::mkdir(b"/dev/shm/sandbox_root\0".as_ptr() as *const c_char, 0o777);
        if libc::mount(
            b"/\0".as_ptr() as *const c_char,
            b"/\0".as_ptr() as *const c_char,
            0 as *const c_char,
            MS_PRIVATE | MS_REC,
            0 as *const libc::c_void,
        ) < 0
        {
            panic!("mount 1 failed");
        }

        // root filesystem
        if libc::mount(
            0 as *const c_char,
            b"/dev/shm/sandbox_root\0".as_ptr() as *const c_char,
            b"tmpfs\0".as_ptr() as *const c_char,
            0,
            0 as *const libc::c_void,
        ) < 0
        {
            panic!("mount 2 failed");
        }

        // read-only access limited to some folders
        mount_dir_p1(b"/dev/shm/sandbox_root/bin\0".as_ptr(), b"/bin\0".as_ptr());
        mount_dir_p1(b"/dev/shm/sandbox_root/etc\0".as_ptr(), b"/etc\0".as_ptr());
        mount_dir_p1(b"/dev/shm/sandbox_root/lib\0".as_ptr(), b"/lib\0".as_ptr());
        mount_dir_p1(
            b"/dev/shm/sandbox_root/lib64\0".as_ptr(),
            b"/lib64\0".as_ptr(),
        );
        mount_dir_p1(b"/dev/shm/sandbox_root/usr\0".as_ptr(), b"/usr\0".as_ptr());

        // access limited to some devices
        // mount_device("/dev/shm/sandbox_root/dev/urandom", "/dev/urandom");
        /*for f in ["/dev/urandom"] {
            let mountpoint = format!("{}{}", destdir, f);
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
        }*/

        // mount new /proc (not in docker)
        libc::mkdir(
            b"/dev/shm/sandbox_root/proc\0".as_ptr() as *const c_char,
            0o777,
        );
        libc::mount(
            0 as *const c_char,
            b"/dev/shm/sandbox_root/proc\0".as_ptr() as *const c_char,
            b"proc\0".as_ptr() as *const c_char,
            0,
            0 as *const c_void,
        );

        // mount data
        mount_dir_p1(
            b"/dev/shm/sandbox_root/data\0".as_ptr(),
            data_folder,
        );

        // chroot jail (try pivot_root first, use classic chroot if not available)
        if pivot_root(
            b"/dev/shm/sandbox_root\0".as_ptr() as *const c_char,
            b"/dev/shm/sandbox_root\0".as_ptr() as *const c_char,
        ) < 0
        {
            libc::chdir(b"/dev/shm/sandbox_root\0".as_ptr() as *const c_char);
            libc::mount(
                b".\0".as_ptr() as *const c_char,
                b"/\0".as_ptr() as *const c_char,
                0 as *const c_char,
                0,
                0 as *const libc::c_void,
            );
            if libc::chroot(b".\0".as_ptr() as *const c_char) < 0 {
                panic!("chroot failed");
            }
        }

        libc::chdir(b"/data\0".as_ptr() as *const c_char);

        // make things actually read-only
        remount_dir(b"/\0".as_ptr());
        remount_dir(b"/bin\0".as_ptr());
        remount_dir(b"/etc\0".as_ptr());
        remount_dir(b"/lib\0".as_ptr());
        remount_dir(b"/lib64\0".as_ptr());
        remount_dir(b"/usr\0".as_ptr());
        remount_dir(b"/data\0".as_ptr());
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

lazy_static! {
    static ref sandbox_mutex: tokio::sync::Mutex<i32> = tokio::sync::Mutex::new(0i32);
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

    // TODO file descriptors should be ok

    // Filesystem and resource limits
    let abs_folder = path::absolute(folder)?;
    unsafe {
        cmd.pre_exec(move || {
            sandbox_mount(my_uid, abs_folder.to_str().unwrap().as_ptr()).expect("cannot mount things");
            sandbox_rlimit().expect("cannot rlimit");

            Ok(())
        });
    }

    let guard = sandbox_mutex.lock().await;
    let mut child = cmd.spawn().expect("failed to spawn");
    drop(guard);

    let wait_result = timeout(runtime, wait_async(&mut child)).await;
    if let Err(_) = wait_result {
        // timeout
        child.kill()?;
        wait_async(&mut child).await?;
    }

    Ok(child)
}

#[cfg(test)]
#[path = "./sandbox_test.rs"]
mod test;
