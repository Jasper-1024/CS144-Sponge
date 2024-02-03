use super::file_descriptor::FileDescriptor;
use nix::fcntl::open;
use nix::fcntl::OFlag;
use nix::ioctl_write_ptr;
use nix::sys::stat::Mode;
use std::ffi::CString;
use std::io::{self, Error, ErrorKind, Result};
use std::os::fd;
use std::os::unix::io::{AsRawFd, RawFd};
use std::ptr;

const TUNSETIFF: u64 = 0x400454ca;
const CLONEDEV: &str = "/dev/net/tun";

ioctl_write_ptr!(tunsetiff, 'T' as u8, 202, libc::ifreq);

// Ifreq结构用于ioctl设置TUN设备名称
#[repr(C)]
#[derive(Debug)]
pub struct Ifreq {
    ifr_name: [u8; 16], // 设备名，例如 tun0
    ifr_flags: i16,     // IFF_TUN 或 IFF_TAP，以及其他可选标志
}

struct TunTapFD {
    fd: FileDescriptor,
    is_tun: bool,
}

impl TunTapFD {
    fn new(devname: &str, is_tun: bool) -> Result<Self> {
        let fd: i32 = open(CLONEDEV, OFlag::O_RDWR, Mode::empty()).map_err(|e| {
            Error::new(
                ErrorKind::NotFound,
                format!("Failed to open /dev/net/tun: {}", e),
            )
        })?;

        let mut ifr = libc::ifreq {
            ifr_ifru: unsafe { std::mem::zeroed() },
            ifr_name: [0; libc::IFNAMSIZ],
        };
        let devname = CString::new(devname)
            .map_err(|_| Error::new(ErrorKind::InvalidInput, "Invalid interface name"))?;
        unsafe {
            ptr::copy_nonoverlapping(
                devname.as_ptr(),
                ifr.ifr_name.as_mut_ptr(),
                devname.to_bytes().len(),
            );
        }

        // Setting flag for TUN or TAP
        ifr.ifr_ifru.ifru_flags = if is_tun {
            (libc::IFF_TUN | libc::IFF_NO_PI).try_into().unwrap()
        } else {
            (libc::IFF_TAP | libc::IFF_NO_PI).try_into().unwrap()
        };

        unsafe {
            tunsetiff(fd, &ifr as *const _ as *mut _).map_err(|e| {
                Error::new(ErrorKind::Other, format!("Failed to set TUNSETIFF: {}", e))
            })?;
        }

        Ok(TunTapFD {
            fd: FileDescriptor::new(fd).unwrap(),
            is_tun,
        })
    }
}

struct TunFD {
    fd: TunTapFD,
}

impl TunFD {
    fn new(devname: &str) -> Result<Self> {
        TunTapFD::new(devname, true).map(|fd| TunFD { fd })
    }
}

struct TapFD {
    fd: TunTapFD,
}

impl TapFD {
    fn new(devname: &str) -> Result<Self> {
        TunTapFD::new(devname, false).map(|fd| TapFD { fd })
    }
}
