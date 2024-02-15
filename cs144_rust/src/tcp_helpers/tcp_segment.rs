use super::tcp_header::TCPHeader;
use crate::{
    tcp_helpers::tcp_header::TCPHeaderTrait,
    util::{
        buffer::{Buffer, BufferList},
        parser::{NetParser, ParseError},
        util::InternetChecksum,
    },
};

pub(crate) struct TCPSegment {
    pub(crate) header: TCPHeader,
    pub(crate) payload: Buffer, // a warp of Rc<[u8]>
}

impl TCPSegment {
    pub fn new(header: TCPHeader, payload: Buffer) -> Self {
        TCPSegment { header, payload }
    }

    pub fn get_payload(&self) -> &Buffer {
        &self.payload
    }

    pub fn get_payload_mut(&mut self) -> &mut Buffer {
        &mut self.payload
    }

    pub fn get_header(&self) -> &TCPHeader {
        &self.header
    }

    pub fn get_header_mut(&mut self) -> &mut TCPHeader {
        &mut self.header
    }
}

pub trait TCPSegmentTrait {
    fn parse(&mut self, p: &mut Buffer, datagram_layer_checksum: u32) -> Result<(), ParseError>; // Parse the segment from a [u8]
    fn serialize(&mut self, datagram_layer_checksum: u32) -> Result<BufferList, &'static str>; // Serialize the segment to a [u8]
    fn length_in_sequence_space(&self) -> usize; // Segment's length in sequence space ,Equal to payload length plus one byte if SYN is set, plus one byte if FIN is set
}

impl TCPSegmentTrait for TCPSegment {
    fn parse(
        &mut self,
        buffer: &mut Buffer,
        datagram_layer_checksum: u32,
    ) -> Result<(), ParseError> {
        let mut check = InternetChecksum::new(datagram_layer_checksum);
        check.add(buffer.as_ref());

        if check.value() != 0 {
            return Err(ParseError::BadChecksum);
        }

        let mut p = NetParser::new(buffer.clone());
        self.header.parse(&mut p)?;
        self.payload = p.buffer();

        Ok(())
    }

    fn serialize(&mut self, datagram_layer_checksum: u32) -> Result<BufferList, &'static str> {
        self.header.cksum = 0;

        let mut check = InternetChecksum::new(datagram_layer_checksum);
        check.add(self.header.serialize()?.as_ref());
        check.add(self.payload.as_ref());

        self.header.cksum = check.value();

        let mut ret = BufferList::new();

        ret.append_vec(self.header.serialize()?);
        ret.append_buffer(self.payload.clone());

        Ok(ret)
    }

    fn length_in_sequence_space(&self) -> usize {
        self.payload.len()
            + if self.header.syn { 1 } else { 0 }
            + if self.header.fin { 1 } else { 0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tcp_helpers::tcp_header::TCPHeader;
    use crate::util::buffer::Buffer;
    use crate::util::parser::ParseError;
    use crate::wrapping_integers::WrappingInt32;

    #[test]
    fn test_get_payload() {
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
        let payload = Buffer::new([
            0x01, 0x02, // source port
            0x03, 0x04, // destination port
            0x05, 0x06, 0x07, 0x08, // sequence number
            0x09, 0x0A, 0x0B, 0x0C, // ack number
            0x50, // data offset
            0x2B, // flags
            0x0D, 0x0E, // window size
            0x0F, 0x10, // checksum
            0x11, 0x12, // urgent pointer
        ]);
        let segment = TCPSegment::new(header, payload);

        assert_eq!(segment.get_payload().len(), 20);
    }

    // 这里测试还存在争议, 需要回顾
    #[test]
    fn test_parse() {
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

        let payload = Buffer::new([]);
        let mut segment = TCPSegment::new(header, payload);

        let mut buffer = Buffer::new([
            0x01, 0x02, // source port
            0x03, 0x04, // destination port
            0x05, 0x06, 0x07, 0x08, // sequence number
            0x09, 0x0A, 0x0B, 0x0C, // ack number
            0x50, // data offset
            0x2B, // flags
            0x0D, 0x0E, // window size
            0x0F, 0x10, // checksum
            0x11, 0x12, // urgent pointer
        ]);

        let a = segment.parse(&mut buffer, 0);

        assert!(a.is_ok());
    }

    // #[test]
    // fn test_serialize() {
    //     let mut segment = TCPSegment::new(TCPHeader::new(), Buffer::new());
    //     let checksum = 0;

    //     assert!(segment.serialize(checksum).is_ok());
    // }

    // #[test]
    // fn test_length_in_sequence_space() {
    //     let header = TCPHeader::new();
    //     let payload = Buffer::new();
    //     let segment = TCPSegment::new(header, payload);

    //     assert_eq!(segment.length_in_sequence_space(), 0);
    // }
}
