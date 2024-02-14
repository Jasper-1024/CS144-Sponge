use libc::writev;
use nix::fcntl::{fcntl, FcntlArg, OFlag};
use nix::libc;
use nix::unistd::{close, read, write};
use std::cell::RefCell;
use std::io::{self, ErrorKind, IoSlice};
use std::os::unix::io::{AsRawFd, RawFd};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use super::buffer::{BufferList, BufferViewList};

// 封装FD的结构体
#[derive(Debug)]
struct FDWrapper {
    fd: RawFd,    // create by kernel
    eof: bool,    // end of file flag
    closed: bool, // close flag
}

// 实现FDWrapper
impl FDWrapper {
    fn new(fd: RawFd) -> io::Result<Self> {
        if fd < 0 {
            Err(io::Error::new(ErrorKind::Other, "Invalid FD number"))
        } else {
            Ok(Self {
                fd,
                eof: false,
                closed: false,
            })
        }
    }

    fn close(&mut self) {
        if !self.closed {
            let res = close(self.fd);
            if let Err(e) = res {
                eprintln!("Failed to close FD: {}", e);
            }
            self.closed = true;
        }
    }
}

// 当FDWrapper实例drop时关闭FD
impl Drop for FDWrapper {
    fn drop(&mut self) {
        self.close();
    }
}

// 共享FDWrapper，提供读写功能
#[derive(Clone)]
pub(crate) struct FileDescriptor {
    internal_fd: Rc<RefCell<FDWrapper>>, // 单线程内部可变
}

impl FileDescriptor {
    pub(crate) fn new(fd: RawFd) -> io::Result<Self> {
        let fd_wrapper = FDWrapper::new(fd)?;
        Ok(Self {
            internal_fd: Rc::new(RefCell::new(fd_wrapper)),
        })
    }

    pub fn fd_num(&self) -> RawFd {
        self.internal_fd.borrow().fd
    }

    pub fn read(&self, mut limit: usize) -> io::Result<Vec<u8>> {
        if limit > (1024 * 1024) {
            limit = 1024 * 1024;
        }
        let mut buffer = vec![0u8; limit];
        let mut guard = self.internal_fd.borrow_mut();

        if guard.closed {
            Err(io::Error::new(ErrorKind::Other, "FD is closed"))
        } else {
            match read(guard.fd, &mut buffer) {
                Ok(bytes_read) => {
                    buffer.truncate(bytes_read);
                    Ok(buffer)
                }
                Err(e) => Err(io::Error::new(ErrorKind::Other, e)),
            }
        }
    }

    pub fn write(&self, buffer: &mut BufferViewList, write_all: bool) -> io::Result<usize> {
        let mut total_bytes_written = 0;

        while !buffer.is_empty() {
            let iovecs = buffer.as_io_slices();

            let bytes_written = unsafe {
                writev(
                    self.internal_fd.borrow().fd,
                    iovecs.as_ptr() as *const libc::iovec,
                    iovecs.len() as libc::c_int,
                )
            };

            if bytes_written < 0 {
                return Err(io::Error::last_os_error());
            }

            let bytes_written = bytes_written as usize;

            if bytes_written == 0 && !buffer.is_empty() {
                return Err(io::Error::new(
                    io::ErrorKind::WriteZero,
                    "write returned 0 given non-empty input buffer",
                ));
            }

            if bytes_written > buffer.len() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "write wrote more than length of input buffer",
                ));
            }

            let _ = buffer.remove_prefix(bytes_written);
            total_bytes_written += bytes_written;

            if !write_all {
                break;
            }
        }

        Ok(total_bytes_written)
    }

    pub fn close(&self) {
        let mut guard = self.internal_fd.borrow_mut();
        guard.close();
    }

    pub fn set_blocking(&self, blocking: bool) -> io::Result<()> {
        let guard = self.internal_fd.borrow();
        let current_flags = fcntl(guard.fd, FcntlArg::F_GETFL)
            .map_err(|e| io::Error::new(ErrorKind::Other, e.to_string()))?;
        let new_flags = if blocking {
            current_flags & !OFlag::O_NONBLOCK.bits()
        } else {
            current_flags | OFlag::O_NONBLOCK.bits()
        };
        fcntl(
            guard.fd,
            FcntlArg::F_SETFL(OFlag::from_bits_truncate(new_flags)),
        )
        .map_err(|e| io::Error::new(ErrorKind::Other, e.to_string()))
        .map(|_| ())
    }

    pub fn duplicate(&self) -> io::Result<Self> {
        let guard = self.internal_fd.clone();
        Ok(Self { internal_fd: guard })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nix::unistd::{pipe, write};
    use std::io::ErrorKind;

    #[test]
    fn test_fdwrapper_new() {
        let fd = FDWrapper::new(-1);
        assert!(fd.is_err());
        assert_eq!(fd.unwrap_err().kind(), ErrorKind::Other);
    }

    #[test]
    fn test_file_descriptor_read_write() {
        let (read_fd, write_fd) = pipe().unwrap();
        let fd = FileDescriptor::new(read_fd).unwrap();
        let write_bytes = write(write_fd, b"Hello, world!").unwrap();
        close(write_fd).unwrap();
        let read_bytes = fd.read(write_bytes).unwrap();
        assert_eq!(read_bytes, b"Hello, world!");
    }

    #[test]
    fn test_file_descriptor_set_blocking() {
        let (read_fd, _) = pipe().unwrap();
        let fd = FileDescriptor::new(read_fd).unwrap();
        assert!(fd.set_blocking(true).is_ok());
        assert!(fd.set_blocking(false).is_ok());
    }
}
