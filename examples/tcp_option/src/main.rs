#![feature(ip_bits)]

use std::net::Ipv4Addr;
use std::os::fd::AsFd;
use std::os::fd::IntoRawFd;
use std::os::unix::fs::OpenOptionsExt;
use std::sync::Arc;
use std::time::Duration;

use libc::socket;
use libc::AF_PACKET;
use libc::SOCK_CLOEXEC;
use libc::SOCK_NONBLOCK;
use libc::SOCK_RAW;

use libc::c_int;
use libc::c_void;
use libc::socklen_t;
use std::mem::size_of_val;

use anyhow::bail;
use anyhow::Ok;
use anyhow::Result;
use clap::Parser;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

use libbpf_rs::skel::OpenSkel;
use libbpf_rs::skel::SkelBuilder;

mod tcp_option {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/bpf/tcp_option.skel.rs"
    ));
}
use tcp_option::*;

const SOL_SOCKET: c_int = 1;
const SO_ATTACH_BPF: c_int = 50;
const ETH_P_ALL: u16 = 0x0003;

#[derive(Debug, Parser)]
struct Command {
    #[arg(short, long)]
    ip: Option<Ipv4Addr>,

    #[arg(short, long, default_value_t = 42)]
    trace_id: u32,

    #[arg(short, long)]
    verbose: bool,
}

fn bump_memlock_rlimit() -> Result<()> {
    let rlimit = libc::rlimit {
        rlim_cur: 128 << 20,
        rlim_max: 128 << 20,
    };

    if unsafe { libc::setrlimit(libc::RLIMIT_MEMLOCK, &rlimit) } != 0 {
        bail!("Failed to increase rlimit");
    }

    Ok(())
}

fn open_fd() -> Result<i32> {
    unsafe {
        match socket(
            AF_PACKET,
            SOCK_RAW | SOCK_NONBLOCK | SOCK_CLOEXEC,
            ETH_P_ALL.to_be() as i32,
        ) {
            -1 => Err(std::io::Error::last_os_error().into()),
            fd => Ok(fd),
        }
    }
}

fn main() -> Result<()> {
    let opts = Command::parse();
    bump_memlock_rlimit()?;

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })?;

    let ip = if let Some(ip) = opts.ip {
        ip.to_bits().to_be()
    } else {
        Ipv4Addr::new(0, 0, 0, 0).to_bits().to_be()
    };

    let mut builder = TcpOptionSkelBuilder::default();
    if opts.verbose {
        builder.obj_builder.debug(true);
    }
    let mut open = builder.open()?;

    open.rodata_mut().targ_ip = ip;
    open.rodata_mut().data_such_as_trace_id = opts.trace_id;

    let mut skel = open.load()?;

    let cgroup_fd = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_DIRECTORY)
        .open("/sys/fs/cgroup")
        .unwrap()
        .into_raw_fd();

    let _kprobe = skel
        .progs_mut()
        .sockops_write_tcp_options()
        .attach_cgroup(cgroup_fd)
        .unwrap();

    let target_socket_fd = open_fd()?;
    let progs = skel.progs();
    let socket_handler = progs.socket_handler();
    let prog_fd = socket_handler.as_fd();
    let _result = match unsafe {
        libc::setsockopt(
            target_socket_fd as c_int,
            SOL_SOCKET,
            SO_ATTACH_BPF,
            &prog_fd as *const _ as *const c_void,
            size_of_val(&prog_fd) as socklen_t,
        )
    } {
        0 => Ok(()),
        _ => Err(std::io::Error::last_os_error().into()),
    };

    while running.load(Ordering::SeqCst) {
        std::thread::sleep(Duration::new(1, 0));
    }
    Ok(())
}
