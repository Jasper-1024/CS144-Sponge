extern crate socket2;
use socket2::{Domain, Protocol, Socket, Type};
use std::{
    io::{self, Read, Write},
    net::ToSocketAddrs,
    process::abort,
};

fn get_url(host: &str, path: &str) -> io::Result<()> {
    let addr = (host, 80)
        .to_socket_addrs()?
        .next()
        .ok_or(io::Error::new(io::ErrorKind::Other, "No address found"))?;
    // ipv4 tcp stream
    let mut sock = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP)).unwrap();

    sock.connect(&addr.into())?;
    sock.write_all(format!("GET {} HTTP/1.1\r\n", path).as_bytes())?;
    sock.write_all(format!("Host: {}\r\n", host).as_bytes())?;
    sock.write_all(format!("Connection: close\r\n\r\n").as_bytes())?;

    let mut buffer = [0u8; 1024];
    loop {
        match sock.read(buffer.as_mut()) {
            Ok(0) => break,
            Ok(n) => {
                print!("{}", String::from_utf8_lossy(&buffer[..n]));
            }
            Err(e) => {
                println!("read error: {}", e);
                break;
            }
        }
    }

    sock.shutdown(std::net::Shutdown::Both)?;
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() <= 0 {
        abort();
    }
    if args.len() != 3 {
        println!("Usage: {} <host> <path>", args[0]);
        abort();
    }
    let host = &args[1];
    let path = &args[2];
    // let host = "cs144.keithw.org";
    // let path = "/hello";

    match get_url(host, path) {
        Ok(_) => {}
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}
