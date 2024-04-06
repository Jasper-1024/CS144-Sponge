use super::file_descriptor::FileDescriptor;
use libc::ifreq;
use nix::fcntl::{open, OFlag};
use nix::ioctl_write_int;
use nix::sys::stat;
use std::ffi::CString;
use std::io;

#[warn(dead_code)]
const CLONEDEV: &str = "/dev/net/tun";
ioctl_write_int!(ioctl_tun_set_iff, b'T', 202);

/// A FileDescriptor to a [Linux TUN/TAP](https://www.kernel.org/doc/Documentation/networking/tuntap.txt) device
struct TunTapFD(FileDescriptor);

impl TunTapFD {
    /// Open an existing persistent [TUN or TAP device](https://www.kernel.org/doc/Documentation/networking/tuntap.txt).
    /// \param[in] devname is the name of the TUN or TAP device, specified at its creation.
    /// \param[in] is_tun is `true` for a TUN device (expects IP datagrams), or `false` for a TAP device (expects Ethernet frames)
    ///
    /// To create a TUN device, you should already have run
    ///
    ///     ip tuntap add mode tun user `username` name `devname`
    ///
    /// as root before calling this function.
    // ref: https://gist.github.com/harrisonturton/f7c8bc2c0396bae08999563840bdf964
    pub fn new(devname: &str, is_tun: bool) -> io::Result<Self> {
        if devname.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Created TunFD with invalid devname",
            ));
        }

        // open the tun device
        let fd_raw = open(CLONEDEV, OFlag::O_RDWR, stat::Mode::empty())?;

        let mut tun_req: ifreq = unsafe { std::mem::zeroed() };

        tun_req.ifr_ifru.ifru_flags = if is_tun {
            (libc::IFF_TUN | libc::IFF_NO_PI) as i16 // tun device with no packetinfo
        } else {
            (libc::IFF_TAP | libc::IFF_NO_PI) as i16
        };

        let cstring_devname = CString::new(devname).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "Device name contains a null character",
            )
        })?;

        // Copy the device name into ifr_name and ensure it's null-terminated
        let bytes = cstring_devname.as_bytes_with_nul();
        let bytes_c_char: &[libc::c_char] =
            unsafe { &*(bytes as *const [u8] as *const [libc::c_char]) };
        tun_req.ifr_name[..bytes.len()].copy_from_slice(bytes_c_char);

        // Perform the ioctl to create and configure the TUN/TAP device
        unsafe {
            ioctl_tun_set_iff(fd_raw, &tun_req as *const _ as u64)?;
        }

        Ok(TunTapFD(FileDescriptor::new_from_rawfd(fd_raw)?))
    }
}

/// A FileDescriptor to a [Linux TUN](https://www.kernel.org/doc/Documentation/networking/tuntap.txt) device
struct TunFD(TunTapFD);

impl TunFD {
    fn new(devname: &str) -> io::Result<Self> {
        print!("{}", devname);
        Ok(TunFD(TunTapFD::new(devname, true)?))
    }
}

/// A FileDescriptor to a [Linux TAP](https://www.kernel.org/doc/Documentation/networking/tuntap.txt) device
struct TapFD(TunTapFD);

impl TapFD {
    fn new(devname: &str) -> io::Result<Self> {
        print!("{}", devname);
        Ok(TapFD(TunTapFD::new(devname, false)?))
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
