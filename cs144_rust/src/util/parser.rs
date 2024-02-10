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

// impl NetParser {
//     pub fn new(buffer: Buffer) -> Self {
//         NetParser {
//             buffer: Arc::new(buffer),
//             error: None,
//         }
//     }

//     pub fn check_size(&mut self, size: usize) {
//         if self.buffer.len() < size {
//             self.error = Some(ParseResult::PacketTooShort);
//         }
//     }

//     pub fn parse_int<T>(&mut self) -> Option<T>
//     where
//         T: std::convert::TryFrom<u128> + std::marker::Sized,
//     {
//         self.check_size(std::mem::size_of::<T>());
//         if self.error.is_some() {
//             return None;
//         }

//         let mut val: u128 = 0;
//         for _ in 0..std::mem::size_of::<T>() {
//             if let Some(byte) = self.buffer.as_bytes().get(0) {
//                 // Assuming Buffer has as_bytes method
//                 val = (val << 8) | (*byte as u128);
//                 self.buffer.remove_prefix(1); // Assuming Buffer has remove_prefix method
//             }
//         }
//         T::try_from(val).ok()
//     }

//     pub fn remove_prefix(&mut self, n: usize) {
//         self.buffer.remove_prefix(n);
//     }

//     // Additional methods for parsing specific network protocol fields...
// }

// Unparser would be similarly implemented, using BufferList for constructing messages

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
}
