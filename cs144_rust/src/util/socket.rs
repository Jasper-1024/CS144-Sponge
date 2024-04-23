use libc::{
    c_int, c_void, msghdr, recvfrom, sendmsg, setsockopt, sockaddr, socklen_t, MSG_TRUNC,
    SOL_SOCKET, SO_DOMAIN, SO_REUSEADDR, SO_TYPE,
};
use nix::sys::socket::{socket, AddressFamily, SockFlag, SockType};

use super::address::{Address, AddressTrait};
use super::buffer::BufferViewList;
use super::file_descriptor::{FileDescriptor, FileDescriptorTrait};
use std::io;
use std::net::Shutdown;
use std::ops::{Deref, DerefMut};
use std::os::fd::{AsRawFd, RawFd};

extern crate libc;

/// Base class for network sockets (TCP, UDP, etc.)
/// Socket is generally used via a subclass. See TCPSocket and UDPSocket for usage examples.
pub struct Socket {
    fd: FileDescriptor,
}

impl Socket {
    /// default constructor for socket of (subclassed) domain and type
    /// \param[in] domain is as described in [socket(7)](\ref man7::socket), probably `AF_INET` or `AF_UNIX`
    /// \param[in] type is as described in [socket(7)](\ref man7::socket)
    pub fn new(domain: AddressFamily, type_: SockType) -> io::Result<Self> {
        let flags = SockFlag::empty();
        let socket = socket(domain, type_, flags, None)?;
        let fd = FileDescriptor::new(socket)?;
        Ok(Self { fd })
    }
    /// construct from file descriptor
    /// \param[in] fd is the FileDescriptor from which to construct
    /// \param[in] domain is `fd`'s domain; throws std::runtime_error if wrong value is supplied
    /// \param[in] type is `fd`'s type; throws std::runtime_error if wrong value is supplied
    pub fn new_from_fd(
        fd: &FileDescriptor,
        domain: AddressFamily,
        type_: SockType,
    ) -> io::Result<Self> {
        // Verify domain
        let mut actual_value: isize = 0;
        let mut len: socklen_t = std::mem::size_of::<c_int>() as socklen_t;
        unsafe {
            if libc::getsockopt(
                fd.as_raw_fd(),
                SOL_SOCKET,
                SO_DOMAIN,
                &mut actual_value as *mut _ as *mut _,
                &mut len,
            ) == -1
            {
                return Err(io::Error::last_os_error());
            }
        }

        if len != std::mem::size_of::<c_int>() as u32 || actual_value != domain as isize {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "socket domain mismatch",
            ));
        }

        // Verify type
        len = std::mem::size_of::<c_int>() as socklen_t;

        unsafe {
            if libc::getsockopt(
                fd.as_raw_fd(),
                SOL_SOCKET,
                SO_TYPE,
                &mut actual_value as *mut _ as *mut _,
                &mut len,
            ) == -1
            {
                return Err(io::Error::last_os_error());
            }
        }

        if len != std::mem::size_of::<c_int>() as u32 || actual_value != type_ as isize {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "socket type mismatch",
            ));
        }

        // let actual_type = socket::getsockopt(fd, socket::sockopt::SockType)?;
        // if actual_type != type_ {
        //     return Err(io::Error::new(
        //         io::ErrorKind::InvalidInput,
        //         "socket type mismatch",
        //     ));
        // }

        Ok(Self { fd: fd.clone() })
    }

    /// set socket option
    /// \param[in] level The protocol level at which the argument resides
    /// \param[in] option A single option to set
    /// \param[in] option_value The value to set
    /// \details See [setsockopt(2)](\ref man2::setsockopt) for details.
    pub fn setsockopt<T>(&self, level: c_int, option: c_int, option_value: &T) -> io::Result<()> {
        let ret = unsafe {
            setsockopt(
                self.fd.as_raw_fd(),
                level,
                option,
                option_value as *const _ as *const _,
                std::mem::size_of::<T>() as libc::socklen_t,
            )
        };
        if ret == -1 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }
}

impl Deref for Socket {
    type Target = FileDescriptor;
    fn deref(&self) -> &Self::Target {
        &self.fd
    }
}

impl DerefMut for Socket {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.fd
    }
}

pub trait SocketTrait {
    // Bind a socket to a specified address with [bind(2)](\ref man2::bind), usually for listen/accept
    fn bind(&self, addr: &Address) -> io::Result<()>;
    // Connect a socket to a specified peer address with [connect(2)](\ref man2::connect)
    fn connect(&self, addr: &Address) -> io::Result<()>;
    // Shut down a socket via [shutdown(2)](\ref man2::shutdown)
    fn shutdown(&self, how: Shutdown) -> io::Result<()>;
    // Get local address of socket with [getsockname(2)](\ref man2::getsockname)
    fn local_address(&self) -> io::Result<Address>;
    // Get peer address of socket with [getpeername(2)](\ref man2::getpeername)
    fn peer_address(&self) -> io::Result<Address>;
    // Allow local address to be reused sooner via [SO_REUSEADDR](\ref man7::socket)
    fn set_reuseaddr(&self) -> io::Result<()>;
}

impl SocketTrait for Socket {
    // bind socket to a specified local address (usually to listen/accept)
    // \param[in] address is a local Address to bind
    fn bind(&self, addr: &Address) -> io::Result<()> {
        unsafe {
            if libc::bind(self.fd.as_raw_fd(), addr.to_sockaddr(), addr.size()) == -1 {
                return Err(io::Error::last_os_error());
            }
        }
        Ok(())
    }

    // connect socket to a specified peer address
    // \param[in] address is the peer's Address
    fn connect(&self, addr: &Address) -> io::Result<()> {
        unsafe {
            if libc::connect(self.fd.as_raw_fd(), addr.to_sockaddr(), addr.size()) == -1 {
                return Err(io::Error::last_os_error());
            }
        }
        Ok(())
    }

    // shut down a socket in the specified way
    // \param[in] how can be `SHUT_RD`, `SHUT_WR`, or `SHUT_RDWR`; see [shutdown(2)](\ref man2::shutdown)
    fn shutdown(&self, how: Shutdown) -> io::Result<()> {
        unsafe {
            if libc::shutdown(self.fd.as_raw_fd(), how as c_int) == -1 {
                return Err(io::Error::last_os_error());
            }
        }
        Ok(())
    }
    // \returns the local Address of the socket
    fn local_address(&self) -> io::Result<Address> {
        unsafe {
            let mut addr = Address::default();
            if libc::getsockname(self.fd.as_raw_fd(), addr.to_sockaddr_mut(), &mut addr.len) == -1 {
                return Err(io::Error::last_os_error());
            }
            Ok(addr)
        }
    }
    // \returns the socket's peer's Address
    fn peer_address(&self) -> io::Result<Address> {
        unsafe {
            let mut addr = Address::default();
            if libc::getpeername(self.fd.as_raw_fd(), addr.to_sockaddr_mut(), &mut addr.len) == -1 {
                return Err(io::Error::last_os_error());
            }
            Ok(addr)
        }
    }

    // allow local address to be reused sooner, at the cost of some robustness
    // \note Using `SO_REUSEADDR` may reduce the robustness of your application
    fn set_reuseaddr(&self) -> io::Result<()> {
        let option_value = 1;
        self.setsockopt(SOL_SOCKET, SO_REUSEADDR, &option_value)
    }
}

fn sendmsg_helper(
    fd: RawFd,
    destination_address: Option<&sockaddr>,
    destination_address_len: socklen_t,
    payload: &BufferViewList,
) -> io::Result<()> {
    let iovecs = payload.as_io_slices();

    let mut message: msghdr = unsafe { std::mem::zeroed() };
    message.msg_name =
        destination_address.map_or(std::ptr::null_mut(), |a| a as *const _ as *mut _);
    message.msg_namelen = destination_address_len;
    message.msg_iov = iovecs.as_ptr() as *mut _;
    message.msg_iovlen = iovecs.len();

    let bytes_sent = unsafe { sendmsg(fd, &message, 0) };

    if bytes_sent == -1 {
        return Err(io::Error::last_os_error());
    }

    if bytes_sent as usize != payload.total_size() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "datagram payload too big for sendmsg()",
        ));
    }

    Ok(())
}
/// \class UDPSocket
/// Functions in this class are essentially wrappers over their POSIX eponyms.
pub struct UDPSocket(Socket);

impl UDPSocket {
    /// Default: construct an unbound, unconnected UDP socket
    pub fn new() -> io::Result<Self> {
        let socket = Socket::new(AddressFamily::Inet, SockType::Datagram)?;
        Ok(UDPSocket(socket))
    }
    /// \brief Construct from FileDescriptor (used by TCPOverUDPSocketAdapter)
    /// \param[in] fd is the FileDescriptor from which to construct
    fn new_from_fd(fd: FileDescriptor) -> io::Result<Self> {
        let socket = Socket::new_from_fd(&fd, AddressFamily::Inet, SockType::Datagram)?;
        Ok(UDPSocket(socket))
    }
}

/// Implement Deref for UDPSocket
impl Deref for UDPSocket {
    type Target = Socket;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for UDPSocket {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub trait UDPSocketTrait {
    /// Receive a datagram and the Address of its sender
    /// Here is just a simple wrapper (Address, Vec<u8>)
    fn recv(&self, mtu: usize) -> io::Result<(Address, Vec<u8>)>;
    /// Receive a datagram and the Address of its sender (caller can allocate storage)
    fn recv_into_buffer(
        &self,
        buf: &mut [u8],
        address: &mut Address,
        buf_size: usize,
    ) -> io::Result<isize>;
    /// Send a datagram to specified Address
    fn send_to(&self, destination: &Address, payload: &BufferViewList) -> io::Result<usize>;
    /// Send datagram to the socket's connected address (must call connect() first)
    fn send(&self, payload: &BufferViewList) -> io::Result<usize>;
}

impl UDPSocketTrait for UDPSocket {
    fn recv(&self, mtu: usize) -> io::Result<(Address, Vec<u8>)> {
        let mut address = Address::default();
        let mut payload = vec![0; mtu];
        let recv_len = self.recv_into_buffer(&mut payload, &mut address, mtu)?;

        payload.truncate(recv_len as usize);
        Ok((address, payload))
    }

    fn recv_into_buffer(
        &self,
        buf: &mut [u8],
        address: &mut Address,
        mtu: usize,
    ) -> io::Result<isize> {
        // let mut sockaddr: sockaddr = unsafe { std::mem::zeroed() };
        // let addr_len =

        let recv_len = unsafe {
            recvfrom(
                self.0.fd.as_raw_fd(),
                buf as *mut _ as *mut c_void,
                mtu,
                MSG_TRUNC,
                address.to_sockaddr_mut(),
                // &mut sockaddr,
                &mut address.len as *mut _,
            )
        };

        if recv_len == -1 {
            return Err(io::Error::last_os_error());
        }

        if recv_len > mtu as isize {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "recvfrom (oversized datagram)",
            ));
        }
        Ok(recv_len)
    }

    fn send(&self, payload: &BufferViewList) -> io::Result<usize> {
        sendmsg_helper(self.0.fd.as_raw_fd(), None, 0, payload)?;
        Ok(payload.total_size())
    }
    fn send_to(&self, destination: &Address, payload: &BufferViewList) -> io::Result<usize> {
        sendmsg_helper(
            self.0.fd.as_raw_fd(),
            Some(destination.to_sockaddr()),
            destination.size(),
            payload,
        )?;
        Ok(payload.total_size())
    }
}

/// \class TCPSocket
/// Functions in this class are essentially wrappers over their POSIX eponyms.

pub struct TCPSocket(Socket);

impl TCPSocket {
    /// Default: construct an unbound, unconnected TCP socket
    pub fn new() -> io::Result<Self> {
        let socket = Socket::new(AddressFamily::Inet, SockType::Stream)?;
        Ok(TCPSocket(socket))
    }
    fn new_from_fd(fd: FileDescriptor) -> io::Result<Self> {
        let socket = Socket::new_from_fd(&fd, AddressFamily::Inet, SockType::Stream)?;
        Ok(TCPSocket(socket))
    }
}

/// Implement Deref for TCPSocket
impl Deref for TCPSocket {
    type Target = Socket;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TCPSocket {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub trait TCPSocketTrait {
    /// Mark a socket as listening for incoming connections
    fn listen(&self, backlog: i32) -> io::Result<()>;
    /// Accept a new incoming connection
    fn accept(&self) -> io::Result<TCPSocket>;
}

impl TCPSocketTrait for TCPSocket {
    // mark the socket as listening for incoming connections
    // \param[in] backlog is the number of waiting connections to queue (see [listen(2)](\ref man2::listen))
    fn listen(&self, backlog: i32) -> io::Result<()> {
        unsafe {
            if libc::listen(self.0.fd.as_raw_fd(), backlog) == -1 {
                return Err(io::Error::last_os_error());
            }
        }
        Ok(())
    }
    // accept a new incoming connection
    // \returns a new TCPSocket connected to the peer.
    // \note This function blocks until a new connection is available
    fn accept(&self) -> io::Result<TCPSocket> {
        let new_fd = unsafe {
            libc::accept(
                self.0.fd.as_raw_fd(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        };
        if new_fd == -1 {
            return Err(io::Error::last_os_error());
        }

        let new_fd = FileDescriptor::new_from_rawfd(new_fd)?;
        Self::new_from_fd(new_fd)
    }
}

/// A wrapper around [Unix-domain stream sockets](\ref man7::unix)
pub struct LocalStreamSocket(Socket);

#[allow(unused)]
impl LocalStreamSocket {
    /// Construct from a file descriptor
    fn new_from_fd(fd: FileDescriptor) -> io::Result<Self> {
        let socket = Socket::new_from_fd(&fd, AddressFamily::Unix, SockType::Stream)?;
        Ok(LocalStreamSocket(socket))
    }
}

/// Implement Deref for LocalStreamSocket
impl Deref for LocalStreamSocket {
    type Target = Socket;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for LocalStreamSocket {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// All test from doctests/socket_dt.cc
#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::net::UnixStream;

    /// socket_example_1.cc
    #[test]
    fn test_udp_socket() {
        let portnum = (rand::random::<u16>() % 50000) + 1025;
        // create a UDP socket and bind it to a local address
        let sock1 = UDPSocket::new().unwrap();
        let _ = sock1.bind(&Address::new_ip_port("127.0.0.1", portnum).unwrap());
        // create another UDP socket and send a datagram to the first socket without connecting
        let sock2 = UDPSocket::new().unwrap();
        let _ = sock2.send_to(
            &Address::new_ip_port("127.0.0.1", portnum).unwrap(),
            &BufferViewList::new_from_slice(b"hi there"),
        );
        // receive sent datagram, connect the socket to the peer's address, and send a response
        let recvd = sock1.recv(65536).unwrap();
        let _ = sock1.connect(&recvd.0);
        let _ = sock1.send(&BufferViewList::new_from_slice(b"hi yourself"));

        let recvd2 = sock2.recv(65536).unwrap();

        assert_eq!(recvd.1, "hi there".as_bytes());
        assert_eq!(recvd2.1, "hi yourself".as_bytes());
    }

    /// socket_example_2.cc
    #[test]
    fn test_tcp_socket() {
        let portnum = (rand::random::<u16>() % 50000) + 1025;

        let sock1 = TCPSocket::new().unwrap();
        let _ = sock1.bind(&Address::new_ip_port("127.0.0.1", portnum).unwrap());
        sock1.listen(1).unwrap();

        let mut sock2 = TCPSocket::new().unwrap();
        let _ = sock2.connect(&Address::new_ip_port("127.0.0.1", portnum).unwrap());

        let mut sock3 = sock1.accept().unwrap();
        let _ = sock3.write(b"hi there", true);

        let recvd = sock2.read(65536).unwrap();
        let _ = sock2.write(b"hi yourself", true);

        let recvd2 = sock3.read(65536).unwrap();

        assert_eq!(recvd, b"hi there");
        assert_eq!(recvd2, b"hi yourself");
    }

    /// socket_example_3.cc
    #[test]
    fn test_localstreamsocket() {
        // create a pair of stream sockets
        let (pipe1, pipe2) = UnixStream::pair().expect("socketpair failed");

        let mut pipe1 = LocalStreamSocket::new_from_fd(
            FileDescriptor::new_from_rawfd(pipe1.as_raw_fd()).unwrap(),
        )
        .unwrap();
        let mut pipe2 = LocalStreamSocket::new_from_fd(
            FileDescriptor::new_from_rawfd(pipe2.as_raw_fd()).unwrap(),
        )
        .unwrap();

        let _ = pipe1.write(b"hi there", true);
        let recvd = pipe2.read(65536).unwrap();

        let _ = pipe2.write(b"hi yourself", true);
        let recvd2 = pipe1.read(65536).unwrap();

        assert_eq!(recvd, b"hi there");
        assert_eq!(recvd2, b"hi yourself");
    }
}
