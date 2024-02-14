use socket2::SockAddr;
use std::io;
use std::net::{IpAddr, SocketAddr, ToSocketAddrs};

// use socket2::{SockAddr, Domain, Type, Protocol};
use std::net::{Ipv4Addr, SocketAddrV4};

/// Address 结构体，用于封装socket地址信息
#[derive(Debug)]
pub struct Address {
    pub inner: SockAddr,
}

impl Address {
    /// hostneme, service
    pub fn new(hostname: &str, service: &str) -> io::Result<Self> {
        // 服务名 to port
        let port: u16 = service
            .parse()
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "Invalid service/port"))?;

        // `(&str, u16)` `to_socket_addrs`
        let addr = (hostname, port)
            .to_socket_addrs()?
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "No address found"))?;

        let sock_addr = SockAddr::from(addr);
        Ok(Address { inner: sock_addr })
    }

    /// (ip, port)
    pub fn new_ip_port(ip: &str, port: u16) -> io::Result<Self> {
        let ip_addr = ip.parse::<IpAddr>().map_err(|_| {
            io::Error::new(io::ErrorKind::InvalidInput, "Invalid IP address format")
        })?;
        let sock_addr = SockAddr::from(SocketAddr::new(ip_addr, port));
        Ok(Address { inner: sock_addr })
    }

    /// (sokcet)
    pub fn new_socketaddr(addr: &SocketAddr) -> Self {
        Address {
            inner: SockAddr::from(addr.clone()),
        }
    }

    pub fn new_sockaddr(addr: &SockAddr) -> Self {
        Address {
            inner: addr.clone(),
        }
    }
}
pub trait AddressTrait {
    fn ip_port(&self) -> (String, u16);
    fn ip(&self) -> String;
    fn port(&self) -> u16;
    fn ipv4_numeric(&self) -> u32;
    fn from_ipv4_numeric(ipv4_numeric: u32) -> Self;
    fn to_string(&self) -> String;
    fn to_sock_addr(&self) -> &SockAddr;
}

impl AddressTrait for Address {
    fn ip_port(&self) -> (String, u16) {
        // 解包为 std::net::SocketAddr
        if let Some(socket_addr) = self.inner.as_socket() {
            match socket_addr {
                SocketAddr::V4(v4) => (v4.ip().to_string(), v4.port()),
                SocketAddr::V6(v6) => (v6.ip().to_string(), v6.port()),
            }
        } else {
            panic!("Address is not SocketAddr");
        }
    }

    fn ip(&self) -> String {
        self.ip_port().0
    }

    fn port(&self) -> u16 {
        self.ip_port().1
    }

    fn ipv4_numeric(&self) -> u32 {
        if let Some(v4) = self.inner.as_socket_ipv4() {
            u32::from_be_bytes(v4.ip().octets()) // big endian
        } else {
            panic!("Address is not IPv4");
        }
    }

    fn from_ipv4_numeric(ipv4_numeric: u32) -> Self {
        let ipv4_addr = Ipv4Addr::from(ipv4_numeric);
        let sock_addr = SockAddr::from(SocketAddrV4::new(ipv4_addr, 0));
        Address { inner: sock_addr }
    }

    fn to_string(&self) -> String {
        format!("{}:{}", self.ip(), self.port())
    }

    fn to_sock_addr(&self) -> &SockAddr {
        &self.inner
    }
}

// PartialEq for !=
impl PartialEq for Address {
    fn eq(&self, other: &Self) -> bool {
        let (ip1, port1) = self.ip_port();
        let (ip2, port2) = other.ip_port();
        ip1 == ip2 && port1 == port2
    }
}

// Eq for ==
impl Eq for Address {}

#[cfg(test)]
mod tests {
    use std::net::IpAddr;

    use super::*;

    #[test]
    fn test_new() {
        let address = Address::new("localhost", "8080").unwrap();
        assert_eq!(address.ip(), "127.0.0.1");
        assert_eq!(address.port(), 8080);
    }

    #[test]
    fn test_new_ip_port() {
        let address = Address::new_ip_port("192.168.0.1", 80).unwrap();
        assert_eq!(address.ip(), "192.168.0.1");
        assert_eq!(address.port(), 80);
    }

    #[test]
    fn test_new_sockaddr() {
        let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let address = Address::new_socketaddr(&socket_addr);
        assert_eq!(address.ip(), "127.0.0.1");
        assert_eq!(address.port(), 8080);
    }

    #[test]
    fn test_ip_port() {
        let address = Address::new("localhost", "8080").unwrap();
        assert_eq!(address.ip_port(), ("127.0.0.1".to_string(), 8080));
    }

    #[test]
    fn test_ipv4_numeric() {
        let address = Address::new("localhost", "8080").unwrap();
        assert_eq!(address.ipv4_numeric(), 2130706433);
    }

    #[test]
    fn test_from_ipv4_numeric() {
        let address = Address::from_ipv4_numeric(2130706433);
        assert_eq!(address.ip(), "127.0.0.1");
        assert_eq!(address.port(), 0);
    }

    #[test]
    fn test_to_string() {
        let address = Address::new("localhost", "8080").unwrap();
        assert_eq!(address.to_string(), "127.0.0.1:8080");
    }

    #[test]
    fn test_eq() {
        let address1 = Address::new("localhost", "8080").unwrap();
        let address2 = Address::new("127.0.0.1", "8080").unwrap();
        assert_eq!(address1, address2);
    }
}
