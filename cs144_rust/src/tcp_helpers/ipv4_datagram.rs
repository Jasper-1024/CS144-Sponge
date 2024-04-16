use super::ipv4_header::IPv4Header;
use crate::{
    tcp_helpers::ipv4_header::IPv4HeaderTrait,
    util::{
        buffer::{Buffer, BufferList},
        parser::{NetParser, ParseError},
        util::InternetChecksum,
    },
};

pub struct IPv4Datagram {
    pub header: IPv4Header,
    pub payload: BufferList,
}

impl Default for IPv4Datagram {
    fn default() -> Self {
        IPv4Datagram {
            header: IPv4Header::default(),
            payload: BufferList::default(),
        }
    }
}

impl IPv4Datagram {
    pub fn new(header: IPv4Header, payload: BufferList) -> Self {
        IPv4Datagram { header, payload }
    }
}

pub trait IPv4DatagramTrait {
    /// \brief Parse the segment from [u8]
    fn parse(&mut self, p: &mut Buffer) -> Result<(), ParseError>;
    /// Serialize the segment to [u8]
    fn serialize(&mut self) -> Result<BufferList, &'static str>;
}

impl IPv4DatagramTrait for IPv4Datagram {
    /// \param[in] buffer string/Buffer to be parsed
    fn parse(&mut self, buffer: &mut Buffer) -> Result<(), ParseError> {
        let mut p = NetParser::new(buffer.clone());
        self.header.parse(&mut p)?;
        self.payload = BufferList::new_from_buffer(p.buffer());

        if self.payload.total_size() != self.header.payload_length() as usize {
            return Err(ParseError::PacketTooShort);
        }
        Ok(())
    }

    fn serialize(&mut self) -> Result<BufferList, &'static str> {
        if self.payload.total_size() != self.header.payload_length() as usize {
            return Err("IPv4Datagram::serialize: payload is wrong size");
        }

        self.header.cksum = 0;
        let header_zero_checksum = self.header.serialize()?;
        
        // calculate checksum -- taken over header only
        let mut check = InternetChecksum::new(0);
        check.add(&header_zero_checksum);
        self.header.cksum = check.value();

        let mut ret = BufferList::new();
        ret.append_vec(self.header.serialize()?);
        ret.append(&mut self.payload.clone());

        Ok(ret)
    }
}
