use std::{
    io,
    ops::{Deref, DerefMut},
};

use crate::{
    tcp_helpers::tcp_segment::TCPSegmentTrait,
    util::{
        address::AddressTrait,
        buffer::{Buffer, BufferViewList},
        socket::{UDPSocket, UDPSocketTrait},
    },
};

use super::{tcp_config::FdAdapterConfig, tcp_segment::TCPSegment};

/// \brief Basic functionality for file descriptor adaptors
/// \details See TCPOverUDPSocketAdapter and TCPOverIPv4OverTunFdAdapter for more information.
struct FdAdapterBase {
    cfg: FdAdapterConfig, //  Configuration values
    listen: bool,         // Is the connected TCP FSM in listen state?
}

impl FdAdapterBase {
    fn new(cfg: FdAdapterConfig) -> Self {
        FdAdapterBase {
            cfg: cfg,
            listen: false,
        }
    }

    /// \brief Set the listening flag
    /// \param[in] l is the new value for the flag
    fn set_listening(&mut self, l: bool) {
        self.listen = l;
    }

    /// \brief Get the listening flag
    /// \returns whether the FdAdapter is listening for a new connection
    fn listening(&self) -> bool {
        self.listen
    }

    /// \brief Get the current configuration
    /// \returns a const reference
    fn config(&self) -> &FdAdapterConfig {
        &self.cfg
    }
    /// \brief Get the current configuration (mutable)
    /// \returns a mutable reference
    fn config_mut(&mut self) -> &mut FdAdapterConfig {
        &mut self.cfg
    }

    /// Called periodically when time elapses
    fn tick(&mut self) {
        // Do nothing by default
    }
}

///  A FD adaptor that reads and writes TCP segments in UDP payloads
struct TCPOverUDPSocketAdapter {
    base: FdAdapterBase,
    sock: UDPSocket,
}

impl Deref for TCPOverUDPSocketAdapter {
    type Target = FdAdapterBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for TCPOverUDPSocketAdapter {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl TCPOverUDPSocketAdapter {
    // fn new(sock: UDPSocket) -> Self {
    //     TCPOverUDPSocketAdapter {
    //         base: FdAdapterBase::new(cfg),
    //         sock: sock,
    //     }
    // }
    pub fn as_socket(&mut self) -> &mut UDPSocket {
        &mut self.sock
    }

    // Access the underlying UDP socket
    pub fn as_socket_const(&self) -> &UDPSocket {
        &self.sock
    }
}

pub trait TCPOverUDPSocketAdapterTrait {
    // Attempts to read and return a TCP segment related to the current connection from a UDP payload
    fn recv(&mut self) -> io::Result<TCPSegment>;
    // Writes a TCP segment into a UDP payload
    fn send(&mut self, seg: &mut TCPSegment) -> io::Result<()>;
}

impl TCPOverUDPSocketAdapterTrait for TCPOverUDPSocketAdapter {
    /// \details This function first attempts to parse a TCP segment from the next UDP
    /// payload recv()d from the socket.
    ///
    /// If this succeeds, it then checks that the received segment is related to the
    /// current connection. When a TCP connection has been established, this means
    /// checking that the source and destination ports in the TCP header are correct.
    ///
    /// If the TCP FSM is listening (i.e., TCPOverUDPSocketAdapter::_listen is `true`)
    /// and the TCP segment read from the wire includes a SYN, this function clears the
    /// `_listen` flag and calls calls connect() on the underlying UDP socket, with
    /// the result that future outgoing segments go to the sender of the SYN segment.
    /// \returns a std::optional<TCPSegment> that is empty if the segment was invalid or unrelated
    fn recv(&mut self) -> io::Result<TCPSegment> {
        let datagram = self.sock.recv(65536)?;

        // is it for us?
        if !self.listening() && datagram.0 != self.config().destination {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Received a packet from an unexpected source",
            ));
        }

        // is the payload a valid TCP segment?
        let mut seg = TCPSegment::default();
        match seg.parse(&mut Buffer::new_form_vec(datagram.1), 0) {
            Ok(_) => {}
            Err(e) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Failed to parse TCP segment: {}", e),
                ));
            }
        }

        // should we target this source in all future replies?
        if self.listening() {
            if seg.header.syn && !seg.header.rst {
                self.config_mut().destination = datagram.0;
                self.set_listening(false);
            } else {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Received a non-SYN segment while listening",
                ));
            }
        }
        Ok(seg)
    }

    /// Serialize a TCP segment and send it as the payload of a UDP datagram.
    /// \param[in] seg is the TCP segment to write
    fn send(&mut self, seg: &mut TCPSegment) -> io::Result<()> {
        seg.header.sport = self.config().source.port().unwrap();
        seg.header.dport = self.config().destination.port().unwrap();

        self.sock.send_to(
            &self.config().destination,
            &BufferViewList::new(&seg.serialize(0).unwrap()),
        )?;

        Ok(())
    }
}
