use super::buffer::Buffer;

#[derive(thiserror::Error, Debug)]
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
        let bytes: [u8; $size] = $self.buffer.as_str()[..$size]
            .as_bytes()
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

    fn check_size(&self, size: usize) -> Result<(), ParseError> {
        if self.buffer.len() < size {
            Err(ParseError::PacketTooShort.into())
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
}

pub struct NetUnparser;

impl NetUnparser {
    pub fn u32(str: &str) -> u32 {
        let num = str.parse::<u32>().unwrap();
        num.to_be()
    }

    pub fn u16(str: &str) -> u16 {
        let num = str.parse::<u16>().unwrap();
        num.to_be()
    }

    pub fn u8(str: &str) -> u8 {
        str.parse::<u8>().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let buffer = Buffer::new("test".to_owned());
        let parser = NetParser::new(buffer.clone());

        assert_eq!(parser.buffer, buffer);
    }

    #[test]
    fn test_remove_prefix() {
        let mut parser = NetParser::new(Buffer::new("test".to_owned()));

        assert!(parser.remove_prefix(2).is_ok());
        assert_eq!(parser.buffer.as_str(), "st");

        assert!(parser.remove_prefix(2).is_ok());
        assert_eq!(parser.buffer.as_str(), "");

        assert!(parser.remove_prefix(1).is_err());
    }

    #[test]
    fn test_u32() {
        let mut parser = NetParser::new(Buffer::new("\x01\x02\x03\x04".to_owned()));

        let len1 = parser.buffer.len();
        assert_eq!(parser.u32().unwrap(), 0x01020304);
        assert_eq!(parser.buffer.len(), len1 - 4);
    }

    #[test]
    fn test_u16() {
        let mut parser = NetParser::new(Buffer::new("\x01\x02".to_owned()));
        let len1 = parser.buffer.len();
        assert_eq!(parser.u16().unwrap(), 0x0102);
        assert_eq!(parser.buffer.len(), len1 - 2);
    }

    #[test]
    fn test_u8() {
        let mut parser = NetParser::new(Buffer::new("\x01".to_owned()));
        let len1 = parser.buffer.len();
        assert_eq!(parser.u8().unwrap(), 0x01);
        assert_eq!(parser.buffer.len(), len1 - 1);
    }

    // Unparser

    #[test]
    fn test_unparser_u32() {
        assert_eq!(NetUnparser::u32("1234567890"), 1234567890u32.to_be());
        assert_eq!(NetUnparser::u32("4294967295"), 4294967295u32.to_be());
    }

    #[test]
    fn test_unparser_u16() {
        assert_eq!(NetUnparser::u16("12345"), 12345u16.to_be());
        assert_eq!(NetUnparser::u16("65535"), 65535u16.to_be());
    }

    #[test]
    fn test_unparser_u8() {
        assert_eq!(NetUnparser::u8("123"), 123u8);
        assert_eq!(NetUnparser::u8("255"), 255u8);
    }

    #[test]
    #[should_panic(expected = "ParseIntError")]
    fn test_unparser_u32_fail() {
        NetUnparser::u32("4294967296"); // 大于 u32 最大值
    }

    #[test]
    #[should_panic(expected = "ParseIntError")]
    fn test_unparser_u16_fail() {
        NetUnparser::u16("65536"); // 大于 u16 最大值
    }

    #[test]
    #[should_panic(expected = "ParseIntError")]
    fn test_unparser_u8_fail() {
        NetUnparser::u8("256"); // 大于 u8 最大值
    }
}
