use nix::fcntl::{fcntl, FcntlArg, OFlag};

use nix::sys::uio::writev;
use nix::unistd::read;
use std::cell::RefCell;
use std::io::{self, ErrorKind};
use std::os::fd::{AsFd, BorrowedFd, FromRawFd, OwnedFd};
use std::os::unix::io::{AsRawFd, RawFd};
use std::rc::Rc;

use super::buffer::BufferViewList;

// 共享FDWrapper，提供读写功能
#[allow(unused)]
#[derive(Clone)]
pub struct FileDescriptor {
    internal_fd: Rc<RefCell<OwnedFd>>, // 单线程内部可变
    eof: bool,                         // Flag indicating whether FDWrapper::_fd is at EOF
}

#[allow(dead_code)]
impl FileDescriptor {
    pub(crate) fn new(fd: OwnedFd) -> io::Result<Self> {
        Ok(Self {
            internal_fd: Rc::new(RefCell::new(fd)),
            eof: false,
        })
    }
    pub(crate) fn new_from_rawfd(fd: RawFd) -> io::Result<Self> {
        Ok(Self {
            internal_fd: Rc::new(RefCell::new(unsafe { OwnedFd::from_raw_fd(fd) })),
            eof: false,
        })
    }
    pub fn duplicate(&self) -> Self {
        let guard = self.internal_fd.clone();
        Self {
            internal_fd: guard,
            eof: self.eof,
        }
    }
}

impl AsRawFd for FileDescriptor {
    fn as_raw_fd(&self) -> RawFd {
        self.internal_fd.borrow().as_raw_fd()
    }
}

// impl AsFd for FileDescriptor {
//     fn as_fd(&self) -> BorrowedFd {
//         self.internal_fd.borrow().as_fd()
//     }
// }

pub trait FileDescriptorTrait {
    // Read up to `limit` bytes | 不定长度
    fn read(&mut self, limit: usize) -> io::Result<Vec<u8>>;
    // Read up to `limit` bytes into `str` (caller can allocate storage)
    fn read_into(&mut self, buffer: &mut [u8], limit: usize) -> Result<usize, io::Error>;

    // Write a string, possibly blocking until all is written
    fn write(&mut self, buffer: &[u8], write_all: bool) -> Result<usize, io::Error> {
        Self::write_buffer(self, BufferViewList::new_from_slice(buffer), write_all)
    }
    // Write a string, possibly blocking until all is written
    // fn write_string(&mut self, str: String, write_all: bool) -> Result<usize, io::Error> {
    //     Self::write_buffer(self, BufferViewList::new_from_string(str), write_all)
    // }
    // Write a buffer (or list of buffers), possibly blocking until all is written
    fn write_buffer(&mut self, buffer: BufferViewList, write_all: bool)
        -> Result<usize, io::Error>;

    // fn write(&self, buffer: &mut BufferViewList, write_all: bool) -> io::Result<usize>;
    // fn close(&self);
    // fn duplicate(&self) -> io::Result<Self>
    fn set_blocking(&self, blocking: bool) -> io::Result<()>;
    // FDWrapper accessors
    // fn fd_num(&self) -> BorrowedFd<_>;
    fn eof(&self) -> bool;
    // fn closed(&self) -> bool;
}

impl FileDescriptorTrait for FileDescriptor {
    fn read(&mut self, limit: usize) -> io::Result<Vec<u8>> {
        let mut temp = limit;
        if temp > (1024 * 1024) {
            temp = 1024 * 1024;
        }
        let mut buffer = vec![0u8; temp];

        let bytes_read = self.read_into(&mut buffer, limit)?;

        if bytes_read > temp {
            return Err(io::Error::new(
                ErrorKind::Other,
                "read() read more than requested",
            ));
        }
        buffer.truncate(bytes_read);
        Ok(buffer)
    }

    fn read_into(&mut self, buffer: &mut [u8], limit: usize) -> Result<usize, io::Error> {
        let guard = self.internal_fd.borrow_mut();

        match read(guard.as_raw_fd(), buffer) {
            Ok(bytes_read) => {
                if limit > 0 && bytes_read == 0 {
                    self.eof = true;
                }
                Ok(bytes_read)
            }
            Err(e) => Err(io::Error::new(ErrorKind::Other, e)),
        }
    }

    fn write_buffer(
        &mut self,
        mut buffer: BufferViewList,
        write_all: bool,
    ) -> Result<usize, io::Error> {
        let mut total_bytes_written = 0;

        while !buffer.is_empty() {
            let iovecs = buffer.as_io_slices();

            let fd_warp = self.internal_fd.borrow();
            let fd = fd_warp.as_fd();

            match writev(fd, &iovecs) {
                Ok(bytes_written) => {
                    if bytes_written == 0 && buffer.total_size() != 0 {
                        return Err(io::Error::new(
                            io::ErrorKind::WriteZero,
                            "write returned 0 given non-empty input buffer",
                        ));
                    }

                    if bytes_written > buffer.total_size() {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "write wrote more than length of input buffer",
                        ));
                    }

                    let _ = buffer.remove_prefix(bytes_written);
                    total_bytes_written += bytes_written;
                }
                Err(e) => return Err(io::Error::new(io::ErrorKind::Other, e)),
            }

            if !write_all {
                break;
            }
        }
        Ok(total_bytes_written)
    }

    fn set_blocking(&self, blocking_state: bool) -> io::Result<()> {
        let fd = self.internal_fd.borrow().as_raw_fd();
        let mut flags = fcntl(fd, FcntlArg::F_GETFL)?; // 获取文件描述符的状态标志

        // 根据 blocking_state 设置或清除 O_NONBLOCK 标志
        if blocking_state {
            flags = flags & !OFlag::O_NONBLOCK.bits();
        } else {
            flags = flags | OFlag::O_NONBLOCK.bits();
        }

        // 更新文件描述符的状态标志
        fcntl(fd, FcntlArg::F_SETFL(OFlag::from_bits_retain(flags)))?;
        Ok(())
    }

    fn eof(&self) -> bool {
        self.eof
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::{Read, Write};
    use std::os::unix::io::IntoRawFd;

    #[test]
    fn test_file_descriptor_read() {
        // 创建一个临时文件并写入一些数据
        let mut file = File::create("/tmp/testfile").unwrap();
        file.write_all(b"Hello, world!").unwrap();

        let file = File::open("/tmp/testfile").unwrap();
        // 获取文件的原始文件描述符并创建一个 FileDescriptor
        let fd = file.into_raw_fd();
        let mut fd_wrapper = FileDescriptor::new_from_rawfd(fd).unwrap();

        // 读取文件的内容
        let content = fd_wrapper.read(13).unwrap();
        assert_eq!(content, b"Hello, world!");

        // 检查 EOF 标志
        assert_eq!(fd_wrapper.eof(), false);
        let _ = fd_wrapper.read(1).unwrap();
        assert_eq!(fd_wrapper.eof(), true);
    }

    #[test]
    fn test_file_descriptor_write() {
        // 创建一个临时文件
        let file = File::create("/tmp/testfile").unwrap();

        // 获取文件的原始文件描述符并创建一个 FileDescriptor
        let fd = file.into_raw_fd();
        let mut fd_wrapper = FileDescriptor::new_from_rawfd(fd).unwrap();

        // 写入一些数据
        let bytes_written = fd_wrapper.write(b"Hello, world!", true).unwrap();
        assert_eq!(bytes_written, 13);

        // 读取文件的内容并检查是否与写入的数据匹配
        let mut file = File::open("/tmp/testfile").unwrap();
        let mut content = Vec::new();
        file.read_to_end(&mut content).unwrap();
        assert_eq!(content, b"Hello, world!");
    }

    #[test]
    fn test_file_descriptor_set_blocking() {
        // 创建一个临时文件
        let file = File::create("/tmp/testfile").unwrap();

        // 获取文件的原始文件描述符并创建一个 FileDescriptor
        let fd = file.into_raw_fd();
        let fd_wrapper = FileDescriptor::new_from_rawfd(fd).unwrap();

        // 设置文件描述符为非阻塞模式
        fd_wrapper.set_blocking(false).unwrap();

        // check if the file descriptor is in non-blocking mode
        let flags = fcntl(fd, FcntlArg::F_GETFL).unwrap();
        assert_eq!(flags & OFlag::O_NONBLOCK.bits(), OFlag::O_NONBLOCK.bits());
    }
}
