use std::{
    io,
    mem::{size_of, MaybeUninit},
};

use libc::{
    addrinfo, getaddrinfo, sockaddr, sockaddr_storage, socklen_t, AF_INET, AI_ALL, AI_NUMERICHOST,
    AI_NUMERICSERV, NI_MAXHOST, NI_NUMERICHOST, NI_NUMERICSERV,
};

/// Wrapper around [sockaddr_storage](@ref man7::socket).
pub struct RawAddr {
    storage: sockaddr_storage, // A `sockaddr_storage` is enough space to store any socket address (IPv4 or IPv6).
}

#[allow(dead_code)]
impl RawAddr {
    fn new() -> Self {
        RawAddr {
            storage: unsafe { std::mem::zeroed() },
        }
    }

    fn as_sockaddr_ptr(&self) -> *const sockaddr {
        &self.storage as *const _ as *const sockaddr
    }

    fn as_mut_sockaddr_ptr(&mut self) -> *mut sockaddr {
        &mut self.storage as *mut _ as *mut sockaddr // first cast to (*mut _) and then to sockaddr
    }

    fn size() -> socklen_t {
        size_of::<sockaddr_storage>() as socklen_t
    }
}

/// Address represents a wrapper around IPv4 addresses and DNS operations.
pub struct Address {
    addr: RawAddr,
    pub(crate) len: socklen_t,
}

fn make_hints(ai_flags: i32, ai_family: i32) -> addrinfo {
    let mut hints = unsafe { std::mem::zeroed::<addrinfo>() };
    hints.ai_flags = ai_flags;
    hints.ai_family = ai_family;
    hints
}

impl Address {
    // node is the hostname or dotted-quad address
    // service is the port number or service name
    // hints are criteria for resolving the supplied name
    pub fn new(node: &str, service: &str, hints: &addrinfo) -> io::Result<Self> {
        // try to convert the hostname and service to a CString
        let node_cstr = std::ffi::CString::new(node).expect("CString::new failed");
        let service_cstr = std::ffi::CString::new(service).expect("CString::new failed");

        let mut resolved_address: *mut addrinfo = std::ptr::null_mut(); // prepare for the answer

        unsafe {
            // look up the name or names
            let gai_ret = getaddrinfo(
                node_cstr.as_ptr(),
                service_cstr.as_ptr(),
                hints,
                &mut resolved_address,
            );
            if gai_ret != 0 {
                return Err(io::Error::last_os_error());
            }
        }

        if resolved_address.is_null() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "getaddrinfo returned successfully but with no results",
            ));
        }
        let resolved_address = unsafe { &*resolved_address };

        Ok(Self::new_from_sockaddr(
            unsafe { &*resolved_address.ai_addr }, // raw pointer to &sockaddr
            resolved_address.ai_addrlen as usize,
        ))
    }

    pub fn new_from_sockaddr(addr: &sockaddr, size: usize) -> Self {
        if size > size_of::<libc::sockaddr>() {
            panic!("invalid sockaddr size");
        }

        let mut storage = MaybeUninit::<RawAddr>::uninit();
        unsafe {
            std::ptr::copy_nonoverlapping(
                addr,
                storage.as_mut_ptr() as *mut _ as *mut sockaddr,
                size,
            ); // copy to the storage
            Address {
                addr: storage.assume_init(),
                len: size as socklen_t,
            }
        }
    }

    pub fn new_hostname_service(hostname: &str, service: &str) -> io::Result<Self> {
        Self::new(hostname, service, &make_hints(AI_ALL, AF_INET))
    }

    // hostname to resolve
    // \param[in] service name (from `/etc/services`, e.g., "http" is port 80)
    pub fn new_ip_port(ip: &str, port: u16) -> io::Result<Self> {
        // tell getaddrinfo that we don't want to resolve anything
        Self::new(
            ip,
            &port.to_string(),
            &make_hints(AI_NUMERICHOST | AI_NUMERICSERV, AF_INET),
        )
    }

    // Create an Address from a 32-bit raw numeric IP address
    pub fn from_ipv4_numeric(ipv4_numeric: u32) -> Self {
        let mut ipv4_addr = unsafe {
            std::mem::zeroed::<libc::sockaddr_in>() // zero out the memory
        };
        ipv4_addr.sin_family = AF_INET as u16;
        ipv4_addr.sin_addr.s_addr = u32::to_be(ipv4_numeric);

        Self::new_from_sockaddr(
            unsafe { std::mem::transmute::<_, &sockaddr>(&ipv4_addr) },
            size_of::<libc::sockaddr_in>(),
        )
    }
}

// Eq for ==
impl PartialEq for Address {
    fn eq(&self, other: &Self) -> bool {
        if self.len != other.len {
            return false;
        }

        unsafe {
            libc::memcmp(
                self.addr.as_sockaddr_ptr() as *const _,
                other.addr.as_sockaddr_ptr() as *const _,
                self.len as usize,
            ) == 0
        }
    }
}

// Human-readable string, e.g., "8.8.8.8:53".
impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl Default for Address {
    fn default() -> Self {
        Address {
            addr: RawAddr::new(),
            len: RawAddr::size(), // size of sockaddr_storage
        }
    }
}

pub trait AddressTrait {
    // Dotted-quad IP address string ("18.243.0.1") and numeric port.
    fn ip_port(&self) -> Result<(String, u16), Box<dyn std::error::Error>>;
    // Dotted-quad IP address string ("18.243.0.1").
    fn ip(&self) -> Result<String, Box<dyn std::error::Error>>;
    // Numeric port.
    fn port(&self) -> Result<u16, Box<dyn std::error::Error>>;
    // Numeric IP address as an integer (i.e., in [host byte order](\ref man3::byteorder)).
    fn ipv4_numeric(&self) -> Result<u32, &'static str>;
    // Create an Address from a 32-bit raw numeric IP address
    // fn from_ipv4_numeric(ipv4_numeric: u32) -> Self;
    // Human-readable string, e.g., "8.8.8.8:53".
    // fn to_string(&self) -> String;

    // Low-level operations
    // Size of the underlying address storage.
    fn size(&self) -> socklen_t;
    // Const pointer to the underlying socket address storage.
    fn to_sockaddr(&self) -> &sockaddr;
    fn to_sockaddr_mut(&mut self) -> *mut sockaddr;
}

impl AddressTrait for Address {
    fn ip_port(&self) -> Result<(String, u16), Box<dyn std::error::Error>> {
        let mut ip = [0 as libc::c_char; NI_MAXHOST as usize];
        let mut port = [0 as libc::c_char; 32];

        let ret = unsafe {
            libc::getnameinfo(
                self.addr.as_sockaddr_ptr(),
                self.len,
                ip.as_mut_ptr(),
                NI_MAXHOST,
                port.as_mut_ptr(),
                32,
                NI_NUMERICHOST | NI_NUMERICSERV,
            )
        };

        if ret != 0 {
            return Err(format!("getnameinfo failed: {}", ret).into());
        }
        // Convert the C strings to Rust strings.
        let ip_str = unsafe { std::ffi::CStr::from_ptr(ip.as_ptr()).to_str()?.to_owned() }; // create a CStr from a raw pointer // convert the CStr to a Rust string
        let port_num = unsafe {
            std::ffi::CStr::from_ptr(port.as_ptr())
                .to_str()?
                .parse::<u16>()?
        }; // create a CStr from a raw pointer

        Ok((ip_str, port_num))
    }

    fn ip(&self) -> Result<String, Box<dyn std::error::Error>> {
        let (ip, _) = self.ip_port()?;
        Ok(ip)
    }
    fn port(&self) -> Result<u16, Box<dyn std::error::Error>> {
        let (_, port) = self.ip_port()?;
        Ok(port)
    }

    fn ipv4_numeric(&self) -> Result<u32, &'static str> {
        if (self.addr.storage.ss_family as i32 != AF_INET)
            || (self.len as usize != size_of::<libc::sockaddr_in>())
        {
            return Err("ipv4_numeric called on non-IPV4 address");
        }

        let ipv4_addr = unsafe { &*(self.addr.as_sockaddr_ptr() as *const libc::sockaddr_in) };

        Ok(u32::from_be(ipv4_addr.sin_addr.s_addr))
    }

    fn size(&self) -> socklen_t {
        self.len
    }

    fn to_sockaddr(&self) -> &sockaddr {
        unsafe { &*(self.addr.as_sockaddr_ptr() as *const sockaddr) }
    }

    fn to_sockaddr_mut(&mut self) -> *mut sockaddr {
        self.addr.as_mut_sockaddr_ptr()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // test from /cs144-2021 /doctest/address_dt_xxx
    #[test]
    fn address_dt() {
        let google_webserver = Address::new_hostname_service("www.google.com", "https").unwrap();
        let a_dns_server = Address::new_ip_port("18.71.0.151", 53).unwrap();
        let a_dns_server_numeric = a_dns_server.ipv4_numeric();

        assert!(google_webserver.port().unwrap() == 443);
        assert!(a_dns_server_numeric.unwrap() == 0x12470097)
    }

    #[test]
    fn test_new() {
        let address = Address::new_hostname_service("localhost", "8080").unwrap();
        assert_eq!(address.ip().unwrap(), "127.0.0.1");
        assert_eq!(address.port().unwrap(), 8080);
    }

    #[test]
    fn test_new_ip_port() {
        let address = Address::new_ip_port("192.168.0.1", 80).unwrap();
        assert_eq!(address.ip().unwrap(), "192.168.0.1");
        assert_eq!(address.port().unwrap(), 80);
    }

    #[test]
    fn test_new_sockaddr() {
        let socket_addr = libc::sockaddr_in {
            sin_family: libc::AF_INET as u16,
            sin_port: 8080u16.to_be(),
            sin_addr: libc::in_addr {
                s_addr: 2130706433u32.to_be(),
            },
            sin_zero: [0; 8],
        };
        let address = Address::new_from_sockaddr(
            unsafe { std::mem::transmute::<_, &libc::sockaddr>(&socket_addr) },
            std::mem::size_of_val(&socket_addr),
        );
        assert_eq!(address.ip().unwrap(), "127.0.0.1");
        assert_eq!(address.port().unwrap(), 8080);
    }
}
