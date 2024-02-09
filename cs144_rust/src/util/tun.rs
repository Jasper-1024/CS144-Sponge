use super::file_descriptor::FileDescriptor;
use nix::fcntl::{self, OFlag};
use nix::ioctl_write_int;
use nix::sys::stat;
use nix::unistd::close;
use std::ffi::CString;
use std::os::unix::io::AsRawFd;

ioctl_write_int!(ioctl_tun_set_iff, b'T', 202);
struct TunTapFD {
    fd: FileDescriptor,
}

impl TunTapFD {
    // ref: https://gist.github.com/harrisonturton/f7c8bc2c0396bae08999563840bdf964
    pub fn new(devname: &str, is_tun: bool) -> Result<Self, String> {
        fn to_u64_ptr<T>(val: &T) -> u64 {
            val as *const T as u64
        }
        if devname.is_empty() {
            return Err("Device name must not be empty".to_string());
        }
        // Open the clone device with read/write flags
        let fd = fcntl::open("/dev/net/tun", OFlag::O_RDWR, stat::Mode::empty())
            .map_err(|err| format!("Failed to open file with errno {}", err))?;

        // Initialize the ifreq structure for ioctl
        let mut ifr = libc::ifreq {
            ifr_ifru: unsafe { std::mem::zeroed() }, // all set to 0
            ifr_name: [0; libc::IFNAMSIZ],
        };

        // Set the flags for TUN or TAP device
        ifr.ifr_ifru.ifru_flags = if is_tun {
            (libc::IFF_TUN | libc::IFF_NO_PI) as i16
        } else {
            (libc::IFF_TAP | libc::IFF_NO_PI) as i16
        };

        // Copy the device name into ifr_name and ensure it's null-terminated
        let cstring_devname = CString::new(devname).expect("CString::new failed");
        if cstring_devname.as_bytes().len() >= libc::IFNAMSIZ {
            panic!("Device name too long");
        }
        for (i, &byte) in cstring_devname.as_bytes_with_nul().iter().enumerate() {
            ifr.ifr_name[i] = byte as i8;
        }

        // Perform the ioctl to create and configure the TUN/TAP device
        unsafe {
            if let Err(err) = ioctl_tun_set_iff(fd, to_u64_ptr(&ifr)) {
                let _ = close(fd).map_err(|err| format!("Failed to close file with errno {}", err));
                return Err(err.to_string());
            }
        }

        Ok(TunTapFD {
            fd: FileDescriptor::new(fd.as_raw_fd()).unwrap(),
        })
    }
}

struct TunFD {
    fd: TunTapFD,
}

impl TunFD {
    fn new(devname: &str) -> Result<Self, String> {
        print!("{}", devname);
        Ok(TunTapFD::new(devname, true).map(|fd| TunFD { fd })?)
    }
}

struct TapFD {
    fd: TunTapFD,
}

impl TapFD {
    fn new(devname: &str) -> Result<Self, String> {
        Ok(TunTapFD::new(devname, false).map(|fd| TapFD { fd })?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tun_fd_new() {
        let tun_fd = TunFD::new("tun0");
        assert!(tun_fd.is_ok(), "Failed to create TunFD");
    }

    #[test]
    fn test_tap_fd_new() {
        let tap_fd = TapFD::new("tap0");
        assert!(tap_fd.is_ok(), "Failed to create TapFD");
    }

    #[test]
    fn test_invalid_devname() {
        let tun_fd = TunFD::new("");
        print!("{}", tun_fd.is_err());
        assert!(tun_fd.is_err(), "Created TunFD with invalid devname");

        let tap_fd = TapFD::new("");
        assert!(tap_fd.is_err(), "Created TapFD with invalid devname");
    }
}
