use core::fmt;
use std::net::{Ipv4Addr, Ipv6Addr};

use crate::{
    util::parser::{NetParser, NetUnparser, ParseError},
    wrapping_integers::WrappingInt32,
};

// Constants for the TCP header flags
const URG_FLAG: u8 = 0b0010_0000;
const ACK_FLAG: u8 = 0b0001_0000;
const PSH_FLAG: u8 = 0b0000_1000;
const RST_FLAG: u8 = 0b0000_0100;
const SYN_FLAG: u8 = 0b0000_0010;
const FIN_FLAG: u8 = 0b0000_0001;

const TCP_HEADER_LENGTH: usize = 20;
/// \struct TCPHeader
/// ~~~{.txt}
///   0                   1                   2                   3
///   0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
///  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///  |          Source Port          |       Destination Port        |
///  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///  |                        Sequence Number                        |
///  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///  |                    Acknowledgment Number                      |
///  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///  |  Data |           |U|A|P|R|S|F|                               |
///  | Offset| Reserved  |R|C|S|S|Y|I|            Window             |
///  |       |           |G|K|H|T|N|N|                               |
///  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///  |           Checksum            |         Urgent Pointer        |
///  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///  |                    Options                    |    Padding    |
///  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///  |                             data                              |
///  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// ~~~
// TCPHeader struct definition
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct TCPHeader {
    pub sport: u16,           // source port
    pub dport: u16,           // destination port
    pub seqno: WrappingInt32, // sequence number
    pub ackno: WrappingInt32, // ack number
    pub doff: u8,             // data offset
    pub urg: bool,            // urgent flag
    pub ack: bool,            // ack flag
    pub psh: bool,            // push flag
    pub rst: bool,            // rst flag
    pub syn: bool,            // syn flag
    pub fin: bool,            // fin flag
    pub win: u16,             // window size
    pub cksum: u16,           // checksum
    pub uptr: u16,            // urgent pointer
}

impl Default for TCPHeader {
    fn default() -> Self {
        TCPHeader {
            sport: 0,
            dport: 0,
            seqno: WrappingInt32::new(0),
            ackno: WrappingInt32::new(0),
            doff: 0,
            urg: false,
            ack: false,
            psh: false,
            rst: false,
            syn: false,
            fin: false,
            win: 0,
            cksum: 0,
            uptr: 0,
        }
    }
}

pub trait TCPHeaderTrait {
    fn parse(&mut self, p: &mut NetParser) -> Result<(), ParseError>; // Parse the TCP fields from the provided NetParser
    fn serialize(&self) -> Result<Vec<u8>, &'static str>; // Serialize the TCP header into a byte array
    fn summary(&self) -> String; // Return a string containing a human-readable summary of the header
}

// Return a string containing a header in human-readable format
impl fmt::Display for TCPHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "TCP source port: {:x}\nTCP dest port: {:x}\nTCP seqno: 0x{:x}\nTCP ackno: 0x{:x}\nTCP doff: {:x}\nFlags: urg: {} ack: {} psh: {} rst: {} syn: {} fin: {}\nTCP winsize: {:x}\nTCP cksum: {:x}\nTCP uptr: {:x}\n",
            self.sport,
            self.dport,
            self.seqno,
            self.ackno,
            self.doff,
            self.urg,
            self.ack,
            self.psh,
            self.rst,
            self.syn,
            self.fin,
            self.win,
            self.cksum,
            self.uptr
        )
    }
}

impl TCPHeaderTrait for TCPHeader {
    fn parse(&mut self, p: &mut NetParser) -> Result<(), ParseError> {
        self.sport = p.u16()?;
        self.dport = p.u16()?;
        self.seqno = WrappingInt32::new(p.u32()?);
        self.ackno = WrappingInt32::new(p.u32()?);
        self.doff = p.u8()? >> 4;

        let fl_b = p.u8()?;

        self.urg = (fl_b & URG_FLAG) != 0;
        self.ack = (fl_b & ACK_FLAG) != 0;
        self.psh = (fl_b & PSH_FLAG) != 0;
        self.rst = (fl_b & RST_FLAG) != 0;
        self.syn = (fl_b & SYN_FLAG) != 0;
        self.fin = (fl_b & FIN_FLAG) != 0;

        self.win = p.u16()?;
        self.cksum = p.u16()?;
        self.uptr = p.u16()?;

        if self.doff < 5 {
            return Err(ParseError::HeaderTooShort);
        }
        // skip any options or anything extra in the header
        p.remove_prefix(self.doff as usize * 4 - TCP_HEADER_LENGTH)?; // Remove the prefix from the buffer

        Ok(())
    }

    fn serialize(&self) -> Result<Vec<u8>, &'static str> {
        if self.doff < 5 {
            return Err("TCP Header length is greater than 20 bytes");
        }

        let mut buffer = Vec::new();
        NetUnparser::u16(&mut buffer, self.sport);
        NetUnparser::u16(&mut buffer, self.dport);
        NetUnparser::u32(&mut buffer, self.seqno.raw_value());
        NetUnparser::u32(&mut buffer, self.ackno.raw_value());
        NetUnparser::u8(&mut buffer, self.doff << 4);

        let fl_b: u8 = (if self.urg { URG_FLAG } else { 0 })
            | (if self.ack { ACK_FLAG } else { 0 })
            | (if self.psh { PSH_FLAG } else { 0 })
            | (if self.rst { RST_FLAG } else { 0 })
            | (if self.syn { SYN_FLAG } else { 0 })
            | (if self.fin { FIN_FLAG } else { 0 });
        NetUnparser::u8(&mut buffer, fl_b);
        NetUnparser::u16(&mut buffer, self.win);
        NetUnparser::u16(&mut buffer, self.cksum);
        NetUnparser::u16(&mut buffer, self.uptr);

        return Ok(buffer);
    }

    fn summary(&self) -> String {
        format!(
            "Header(flags={}{}{}{},seqno=0x{:x},ack=0x{:x},win=0x{:x})",
            if self.syn { "S" } else { "" },
            if self.ack { "A" } else { "" },
            if self.rst { "R" } else { "" },
            if self.fin { "F" } else { "" },
            self.seqno,
            self.ackno,
            self.win
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::util::buffer::Buffer;

    use super::*;

    #[test]
    fn test_tcp_header_parse() {
        let mut parser = NetParser::new(Buffer::new([
            0x01, 0x02, // source port
            0x03, 0x04, // destination port
            0x05, 0x06, 0x07, 0x08, // sequence number
            0x09, 0x0A, 0x0B, 0x0C, // ack number
            0x50, // data offset
            0x2B, // flags
            0x0D, 0x0E, // window size
            0x0F, 0x10, // checksum
            0x11, 0x12, // urgent pointer
        ]));

        let mut header = TCPHeader {
            sport: 0,
            dport: 0,
            seqno: WrappingInt32::new(0),
            ackno: WrappingInt32::new(0),
            doff: 0,
            urg: false,
            ack: false,
            psh: false,
            rst: false,
            syn: false,
            fin: false,
            win: 0,
            cksum: 0,
            uptr: 0,
        };

        header.parse(&mut parser).unwrap();

        assert_eq!(header.sport, 0x0102);
        assert_eq!(header.dport, 0x0304);
        assert_eq!(header.seqno, WrappingInt32::new(0x05060708));
        assert_eq!(header.ackno, WrappingInt32::new(0x090A0B0C));
        assert_eq!(header.doff, 0x05);
        assert_eq!(header.urg, true);
        assert_eq!(header.ack, false);
        assert_eq!(header.psh, true);
        assert_eq!(header.rst, false);
        assert_eq!(header.syn, true);
        assert_eq!(header.fin, true);
        assert_eq!(header.win, 0x0D0E);
        assert_eq!(header.cksum, 0x0F10);
        assert_eq!(header.uptr, 0x1112);
    }

    #[test]
    fn test_tcp_header_display() {
        let header = TCPHeader {
            sport: 0x0102,
            dport: 0x0304,
            seqno: WrappingInt32::new(0x05060708),
            ackno: WrappingInt32::new(0x090A0B0C),
            doff: 0x05,
            urg: true,
            ack: true,
            psh: false,
            rst: true,
            syn: false,
            fin: true,
            win: 0x0D0E,
            cksum: 0x0F10,
            uptr: 0x1112,
        };

        assert_eq!(
            format!("{}", header),
            "TCP source port: 102\nTCP dest port: 304\nTCP seqno: 0x5060708\nTCP ackno: 0x90a0b0c\nTCP doff: 5\nFlags: urg: true ack: true psh: false rst: true syn: false fin: true\nTCP winsize: d0e\nTCP cksum: f10\nTCP uptr: 1112\n"
        );
    }

    #[test]
    fn test_summary() {
        let header = TCPHeader {
            sport: 0x1234,
            dport: 0x5678,
            seqno: WrappingInt32::new(0x9abcdef0),
            ackno: WrappingInt32::new(0x13579bdf),
            doff: 5,
            urg: true,
            ack: false,
            psh: true,
            rst: false,
            syn: true,
            fin: false,
            win: 0x2468,
            cksum: 0xace0,
            uptr: 0xfdb9,
        };

        assert_eq!(
            header.summary(),
            "Header(flags=S,seqno=0x9abcdef0,ack=0x13579bdf,win=0x2468)"
        );
    }
}
