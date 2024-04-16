use crate::util::parser::{NetParser, NetUnparser, ParseError};

const LENGTH: isize = 20; //< [IPv4](\ref rfc::rfc791) header length, not including options
const DEFAULT_TTL: u8 = 128; // < A reasonable default TTL value
const PROTO_TCP: u8 = 6; //< Protocol number for [tcp](\ref rfc::rfc793)

/// \struct IPv4Header
/// ~~~{.txt}
///   0                   1                   2                   3
///   0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
///  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///  |Version|  IHL  |Type of Service|          Total Length         |
///  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///  |         Identification        |Flags|      Fragment Offset    |
///  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///  |  Time to Live |    Protocol   |         Header Checksum       |
///  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///  |                       Source Address                          |
///  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///  |                    Destination Address                        |
///  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///  |                    Options                    |    Padding    |
///  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// ~~~
#[derive(Debug)]
pub struct IPv4Header {
    pub ver: u8,     // < IP version
    pub hlen: u8,    // < header length (multiples of 32 bits)
    pub tos: u8,     // < type of service
    pub len: u16,    // < total length of packet
    pub id: u16,     // < identification number
    pub df: bool,    // < don't fragment flag
    pub mf: bool,    // < more fragments flag
    pub offset: u16, // < fragment offset field
    pub ttl: u8,     // < time to live field
    pub proto: u8,   // < protocol field
    pub cksum: u16,  // < checksum field
    pub src: u32,    // < src address
    pub dst: u32,    // < dst address
}

impl Default for IPv4Header {
    fn default() -> Self {
        IPv4Header {
            ver: 4,
            hlen: (LENGTH / 4) as u8,
            tos: 0,
            len: 0,
            id: 0,
            df: true,
            mf: false,
            offset: 0,
            ttl: DEFAULT_TTL,
            proto: PROTO_TCP,
            cksum: 0,
            src: 0,
            dst: 0,
        }
    }
}

pub trait IPv4HeaderTrait {
    fn parse(&mut self, p: &mut NetParser) -> Result<(), ParseError>; // Parse the IP fields from the provided NetParser
    fn serialize(&self) -> Result<Vec<u8>, &'static str>; // Serialize the IP fields
    fn payload_length(&self) -> u16; // Length of the payload
    fn pseudo_cksum(&self) -> u32; // [pseudo-header's](\ref rfc::rfc793) contribution to the TCP checksum
    fn summary(&self) -> String; // Return a string containing a human-readable summary of the header
}

impl IPv4HeaderTrait for IPv4Header {
    /// \param[in,out] p is a NetParser from which the IP fields will be extracted
    /// \returns a ParseResult indicating success or the reason for failure
    /// \details It is important to check for (at least) the following potential errors
    ///          (but note that NetParser inherently checks for certain errors;
    ///          use that fact to your advantage!):
    ///
    /// - data stream is too short to contain a header
    /// - wrong IP version number
    /// - the header's `hlen` field is shorter than the minimum allowed
    /// - there is less data in the header than the `doff` field claims
    /// - there is less data in the full datagram than the `len` field claims
    /// - the checksum is bad
    fn parse(&mut self, p: &mut NetParser) -> Result<(), ParseError> {
        let data_size = p.buffer().len();

        if data_size < LENGTH as usize {
            return Err(ParseError::PacketTooShort);
        }

        let first_byte = p.u8()?;
        self.ver = first_byte >> 4; // version
        self.hlen = first_byte & 0x0f; // header length
        self.tos = p.u8()?; // type of service
        self.len = p.u16()?; // total length
        self.id = p.u16()?; // identification

        let fo_val = p.u16()?;
        self.df = (fo_val & 0x4000) != 0; // don't fragment
        self.mf = (fo_val & 0x2000) != 0; // more fragments
        self.offset = fo_val & 0x1fff; // fragment offset

        self.ttl = p.u8()?; // time to live
        self.proto = p.u8()?; // protocol
        self.cksum = p.u16()?; // checksum
        self.src = p.u32()?; // source address
        self.dst = p.u32()?; // destination address

        if data_size < self.hlen as usize * 4 {
            return Err(ParseError::PacketTooShort);
        }
        if self.ver != 4 {
            return Err(ParseError::WrongIPVersion);
        }
        if self.hlen < 5 {
            return Err(ParseError::HeaderTooShort);
        }
        if self.len as usize != data_size {
            return Err(ParseError::TruncatedPacket);
        }

        p.remove_prefix(self.hlen as usize * 4 - LENGTH as usize)?; // Remove the prefix from the buffer
        Ok(())
    }

    fn payload_length(&self) -> u16 {
        self.len - self.hlen as u16 * 4
    }

    /// \details This value is needed when computing the checksum of an encapsulated TCP segment.
    /// ~~~{.txt}
    ///   0      7 8     15 16    23 24    31
    ///  +--------+--------+--------+--------+
    ///  |          source address           |
    ///  +--------+--------+--------+--------+
    ///  |        destination address        |
    ///  +--------+--------+--------+--------+
    ///  |  zero  |protocol|  payload length |
    ///  +--------+--------+--------+--------+
    /// ~~~
    fn pseudo_cksum(&self) -> u32 {
        let mut pcksum = (self.src >> 16) + (self.src & 0xfff);
        pcksum += (self.dst >> 16) + (self.dst & 0xfff);
        pcksum += self.proto as u32 + self.payload_length() as u32;
        return pcksum;
    }

    /// \returns A string with the header's contents
    fn summary(&self) -> String {
        format!(
            "IPv{}, len={}, protocol={}, {}src={}, dst={}",
            self.ver,
            self.len,
            self.proto,
            if self.ttl >= 10 {
                String::new()
            } else {
                format!("ttl={}, ", self.ttl)
            },
            self.src.to_be(),
            self.dst.to_be()
        )
    }
    /// Serialize the IPv4Header to a string (does not recompute the checksum)
    fn serialize(&self) -> Result<Vec<u8>, &'static str> {
        if self.ver != 4 {
            return Err("wrong IP version");
        }

        if (4 * self.hlen as isize) < LENGTH {
            return Err("IP header too short");
        }

        let mut buffer = Vec::new();
        let first_byte = (self.ver << 4) | self.hlen;
        NetUnparser::u8(&mut buffer, first_byte); // version and header length
        NetUnparser::u8(&mut buffer, self.tos); // type of service
        NetUnparser::u16(&mut buffer, self.len); // length
        NetUnparser::u16(&mut buffer, self.id); // id

        let fo_val =
            (if self.df { 0x4000 } else { 0 }) | (if self.mf { 0x2000 } else { 0 }) | self.offset;
        NetUnparser::u16(&mut buffer, fo_val); // flags and offset

        NetUnparser::u8(&mut buffer, self.ttl); // time to live
        NetUnparser::u8(&mut buffer, self.proto); // protocol number
        NetUnparser::u16(&mut buffer, self.cksum); // checksum
        NetUnparser::u32(&mut buffer, self.src); // src address
        NetUnparser::u32(&mut buffer, self.dst); // dst address
        return Ok(buffer);
    }
}
