use nix::fcntl::{fcntl, FcntlArg, OFlag};
use nix::libc;
use nix::unistd::{close, read, write};
use std::io::{self, ErrorKind};
use std::os::unix::io::{AsRawFd, RawFd};
use std::sync::{Arc, Mutex};

// 封装FD的结构体
#[derive(Debug)]
struct FDWrapper {
    fd: RawFd,
    eof: bool,
    closed: bool,
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
            unsafe {
                close(self.fd).unwrap_or(());
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
struct FileDescriptor {
    internal_fd: Arc<Mutex<FDWrapper>>,
}

impl FileDescriptor {
    fn new(fd: RawFd) -> io::Result<Self> {
        let fd_wrapper = FDWrapper::new(fd)?;
        Ok(Self {
            internal_fd: Arc::new(Mutex::new(fd_wrapper)),
        })
    }

    pub fn read(&self, limit: usize) -> io::Result<Vec<u8>> {
        let mut buffer = vec![0u8; limit];
        let mut guard = self.internal_fd.lock().unwrap();

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

    pub fn write(&self, buffer: &[u8]) -> io::Result<usize> {
        let guard = self.internal_fd.lock().unwrap();

        if guard.closed {
            Err(io::Error::new(ErrorKind::Other, "FD is closed"))
        } else {
            write(guard.fd, buffer).map_err(|e| io::Error::new(ErrorKind::Other, e))
        }
    }

    pub fn close(&self) {
        let mut guard = self.internal_fd.lock().unwrap();
        guard.close();
    }

    pub fn set_blocking(&self, blocking: bool) -> io::Result<()> {
        let guard = self.internal_fd.lock().unwrap();
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
