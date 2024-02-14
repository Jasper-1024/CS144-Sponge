use super::address::AddressTrait;
use super::buffer::BufferViewList;
use super::{address::Address, file_descriptor::FileDescriptor};
use socket2::Socket as SysSocket;
use socket2::{Domain, Type};
use std::io;
use std::net::Shutdown;
use std::os::fd::{AsRawFd, FromRawFd};

struct Socket {
    fd: FileDescriptor,
}

pub trait SocketTrait {
    fn bind(&self, addr: &Address) -> io::Result<()>;
    fn connect(&self, addr: &Address) -> io::Result<()>;
    fn shutdown(&self, how: Shutdown) -> io::Result<()>;
    fn local_address(&self) -> io::Result<Address>;
    fn peer_address(&self) -> io::Result<Address>;
    // fn set_reuseaddr(&self) -> io::Result<()>; // 声明后没有任何使用,暂时不实现了.
}

impl Socket {
    pub fn new(domain: Domain, type_: Type) -> io::Result<Self> {
        let socket = SysSocket::new(domain, type_, None)?;
        let fd = FileDescriptor::new(socket.as_raw_fd())?;
        Ok(Self { fd })
    }

    pub fn new_from_fd(fd: &FileDescriptor, domain: Domain, type_: Type) -> io::Result<Self> {
        let fd_num = fd.fd_num();

        let socket: SysSocket = unsafe { SysSocket::from_raw_fd(fd_num) }; // fd 创建 socket

        // verify the socket type and domain
        let actual_domain = socket.local_addr()?.domain();
        let actual_type = socket.r#type()?;

        if actual_domain != domain {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "socket domain mismatch",
            ));
        }
        if actual_type != type_ {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "socket type mismatch",
            ));
        }

        let fd = FileDescriptor::new(socket.as_raw_fd())?;
        Ok(Self { fd })
    }

    // pub fn setsockopt<T>(
    //     &self,
    //     level: i32,
    //     option: libc::c_int,
    //     option_value: &T,
    // ) -> io::Result<()> {
    //     todo!()
    // }
}

impl SocketTrait for Socket {
    fn bind(&self, addr: &Address) -> io::Result<()> {
        let socket = unsafe { SysSocket::from_raw_fd(self.fd.fd_num()) }; // fd 创建 socket
        socket.bind(addr.to_sock_addr())
    }

    fn connect(&self, addr: &Address) -> io::Result<()> {
        let socket = unsafe { SysSocket::from_raw_fd(self.fd.fd_num()) }; // fd 创建 socket
        socket.connect(addr.to_sock_addr())
    }

    fn shutdown(&self, how: Shutdown) -> io::Result<()> {
        let socket = unsafe { SysSocket::from_raw_fd(self.fd.fd_num()) }; // fd 创建 socket
        socket.shutdown(how)
    }

    fn local_address(&self) -> io::Result<Address> {
        let socket = unsafe { SysSocket::from_raw_fd(self.fd.fd_num()) }; // fd 创建 socket
        let a = socket.local_addr()?;
        Ok(Address::new_sockaddr(&a))
    }

    fn peer_address(&self) -> io::Result<Address> {
        let socket = unsafe { SysSocket::from_raw_fd(self.fd.fd_num()) }; // fd 创建 socket
        let a = socket.peer_addr()?;
        Ok(Address::new_sockaddr(&a))
    }
}

// struct ReceivedDatagram {
//     source_address: Address,
//     payload: String,
// }

// pub struct UDPSocket {
//     socket: Socket,
// }

// impl UDPSocket {
//     pub fn new() -> io::Result<Self> {
//         let socket = Socket::new(Domain::IPV4, Type::DGRAM)?;
//         Ok(Self { socket })
//     }

//     pub fn new_from_fd(fd: FileDescriptor) -> io::Result<Self> {
//         let socket = Socket::new_from_fd(fd, Domain::IPV4, Type::DGRAM)?;
//         Ok(Self { socket })
//     }
// }

// pub trait UdpSocketTrait {
//     fn recv(&self, mtu: usize) -> io::Result<ReceivedDatagram>;
//     fn recv_to(&self, datagram: &mut ReceivedDatagram, mtu: usize) -> io::Result<()>;
//     fn send_to(&self, destination: &Address, payload: &BufferViewList) -> io::Result<()>;
//     fn send(&self, payload: &BufferViewList) -> io::Result<()>;
// }

// impl UdpSocketTrait for UDPSocket {
//     fn recv(&self, mtu: usize) -> io::Result<ReceivedDatagram> {
//         todo!()
//     }

//     fn recv_to(&self, datagram: &mut ReceivedDatagram, mtu: usize) -> io::Result<()> {
//         todo!()
//     }

//     fn send_to(&self, destination: &Address, payload: &BufferViewList) -> io::Result<()> {
//         todo!()
//     }

//     fn send(&self, payload: &BufferViewList) -> io::Result<()> {
//         todo!()
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn test_socket_new() {
        let socket = Socket::new(Domain::IPV4, Type::STREAM);
        assert!(socket.is_ok());
    }

    #[test]
    fn test_socket_new_from_fd() {
        let socket = Socket::new(Domain::IPV4, Type::STREAM).unwrap();
        let fd = socket.fd.clone();
        let socket_from_fd = Socket::new_from_fd(fd, Domain::IPV4, Type::STREAM);
        assert!(socket_from_fd.is_ok());
    }

    // #[test]
    // fn test_socket_bind() {
    //     let socket = Socket::new(Domain::IPV4, Type::STREAM).unwrap();
    //     let addr = Address::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    //     assert!(socket.bind(&addr).is_ok());
    // }

    // #[test]
    // fn test_socket_connect() {
    //     let socket = Socket::new(Domain::IPV4, Type::STREAM).unwrap();
    //     let addr = Address::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    //     assert!(socket.connect(&addr).is_ok());
    // }

    // #[test]
    // fn test_socket_shutdown() {
    //     let socket = Socket::new(Domain::IPV4, Type::STREAM).unwrap();
    //     assert!(socket.shutdown(Shutdown::Both).is_ok());
    // }

    // #[test]
    // fn test_socket_local_address() {
    //     let socket = Socket::new(Domain::IPV4, Type::STREAM).unwrap();
    //     let addr = Address::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    //     socket.bind(&addr).unwrap();
    //     assert_eq!(socket.local_address().unwrap(), addr);
    // }

    // #[test]
    // fn test_socket_peer_address() {
    //     let socket = Socket::new(Domain::IPV4, Type::STREAM).unwrap();
    //     let addr = Address::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    //     socket.connect(&addr).unwrap();
    //     assert_eq!(socket.peer_address().unwrap(), addr);
    // }
}
