use std::vec;

use super::buffer::Buffer;

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum ParseError {
    // #[error("Success")] // rust 不需要这个,返回 ok
    // NoError,
    #[error("Bad checksum")]
    BadChecksum,
    #[error("Not enough data to finish parsing")]
    PacketTooShort,
    #[error("Got a version of IP other than 4")]
    WrongIPVersion,
    #[error("Header length is shorter than minimum required")]
    HeaderTooShort,
    #[error("Packet length is shorter than header claims")]
    TruncatedPacket,
    #[error("Packet uses unsupported features")]
    Unsupported,
    #[error("Unknown error")]
    Unknown,
}

impl From<&'static str> for ParseError {
    fn from(s: &'static str) -> Self {
        match s {
            "Bad checksum" => ParseError::BadChecksum,
            "Not enough data to finish parsing" => ParseError::PacketTooShort,
            "Got a version of IP other than 4" => ParseError::WrongIPVersion,
            "Header length is shorter than minimum required" => ParseError::HeaderTooShort,
            "Packet length is shorter than header claims" => ParseError::TruncatedPacket,
            "Packet uses unsupported features" => ParseError::Unsupported,
            _ => ParseError::Unknown,
        }
    }
}

macro_rules! parse_num {
    ($self:ident, $t:ty, $size:expr) => {{
        $self.check_size($size)?;
        let bytes: [u8; $size] = $self.buffer.as_slice().unwrap_or_default()[..$size]
            .try_into()
            .unwrap();
        let _ = $self.remove_prefix($size);
        Ok(<$t>::from_be_bytes(bytes))
    }};
}

pub struct NetParser {
    buffer: Buffer,
}

impl NetParser {
    pub fn new(buffer: Buffer) -> Self {
        Self { buffer }
    }

    pub fn remove_prefix(&mut self, n: usize) -> Result<(), ParseError> {
        let _ = self.check_size(n)?;
        self.buffer.remove_prefix(n)?;
        Ok(())
    }
    // Check that there is sufficient data to parse the next token
    fn check_size(&self, size: usize) -> Result<(), ParseError> {
        if self.buffer.len() < size {
            Err(ParseError::PacketTooShort)
        } else {
            Ok(())
        }
    }

    pub fn u32(&mut self) -> Result<u32, ParseError> {
        parse_num!(self, u32, 4)
    }

    pub fn u16(&mut self) -> Result<u16, ParseError> {
        parse_num!(self, u16, 2)
    }

    pub fn u8(&mut self) -> Result<u8, ParseError> {
        parse_num!(self, u8, 1)
    }

    pub fn buffer(&self) -> Buffer {
        self.buffer.clone()
    }
}

pub struct NetUnparser;

impl NetUnparser {
    pub fn u32(buffer: &mut Vec<u8>, num: u32) {
        buffer.extend_from_slice(&num.to_be_bytes());
    }

    pub fn u16(buffer: &mut Vec<u8>, num: u16) {
        buffer.extend_from_slice(&num.to_be_bytes());
    }

    pub fn u8(buffer: &mut Vec<u8>, num: u8) {
        buffer.push(num);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let buffer = Buffer::new(*b"test");
        let parser = NetParser::new(buffer.clone());

        assert_eq!(parser.buffer, buffer);
    }

    #[test]
    fn test_remove_prefix() {
        let mut parser = NetParser::new(Buffer::new(*b"test"));

        assert!(parser.remove_prefix(2).is_ok());
        assert_eq!(parser.buffer.as_slice(), Some("st".as_bytes()));

        assert!(parser.remove_prefix(2).is_ok());
        assert_eq!(parser.buffer.as_slice(), Some("".as_bytes()));

        assert!(parser.remove_prefix(1).is_err());
    }

    #[test]
    fn test_u32() {
        let mut parser = NetParser::new(Buffer::new(*b"\x01\x02\x03\x04"));

        let len1 = parser.buffer.len();
        assert_eq!(parser.u32().unwrap(), 0x01020304);
        assert_eq!(parser.buffer.len(), len1 - 4);
    }

    #[test]
    fn test_u16() {
        let mut parser = NetParser::new(Buffer::new(*b"\x01\x02"));
        let len1 = parser.buffer.len();
        assert_eq!(parser.u16().unwrap(), 0x0102);
        assert_eq!(parser.buffer.len(), len1 - 2);
    }

    #[test]
    fn test_u8() {
        let mut parser = NetParser::new(Buffer::new(*b"\x01"));
        let len1 = parser.buffer.len();
        assert_eq!(parser.u8().unwrap(), 0x01);
        assert_eq!(parser.buffer.len(), len1 - 1);
    }

    #[test]
    fn test_unparser_u32() {
        let mut buffer = Vec::new();
        NetUnparser::u32(&mut buffer, 0x01020304);
        assert_eq!(buffer, vec![0x01, 0x02, 0x03, 0x04]);

        buffer.clear();
        NetUnparser::u32(&mut buffer, u32::MAX);
        assert_eq!(buffer, vec![0xFF, 0xFF, 0xFF, 0xFF]);

        buffer.clear();
        NetUnparser::u32(&mut buffer, u32::MIN);
        assert_eq!(buffer, vec![0x00, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn test_unparser_u16() {
        let mut buffer = Vec::new();
        NetUnparser::u16(&mut buffer, 0x0102);
        assert_eq!(buffer, vec![0x01, 0x02]);

        buffer.clear();
        NetUnparser::u16(&mut buffer, u16::MAX);
        assert_eq!(buffer, vec![0xFF, 0xFF]);

        buffer.clear();
        NetUnparser::u16(&mut buffer, u16::MIN);
        assert_eq!(buffer, vec![0x00, 0x00]);
    }

    #[test]
    fn test_unparser_u8() {
        let mut buffer = Vec::new();
        NetUnparser::u8(&mut buffer, 0x01);
        assert_eq!(buffer, vec![0x01]);

        buffer.clear();
        NetUnparser::u8(&mut buffer, u8::MAX);
        assert_eq!(buffer, vec![0xFF]);

        buffer.clear();
        NetUnparser::u8(&mut buffer, u8::MIN);
        assert_eq!(buffer, vec![0x00]);
    }
}
