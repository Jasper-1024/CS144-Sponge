use std::net::{Ipv4Addr, Ipv6Addr};

use crate::wrapping_integers::WrappingInt32;

// Constants for the TCP header flags
const URG_FLAG: u8 = 0b0010_0000;
const ACK_FLAG: u8 = 0b0001_0000;
const PSH_FLAG: u8 = 0b0000_1000;
const RST_FLAG: u8 = 0b0000_0100;
const SYN_FLAG: u8 = 0b0000_0010;
const FIN_FLAG: u8 = 0b0000_0001;

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
    pub sport: u16,         // source port
    pub dport: u16,         // destination port
    pub seqno: WrappingInt32, // sequence number
    pub ackno: WrappingInt32, // ack number
    pub doff: u8,           // data offset
    pub urg: bool,          // urgent flag
    pub ack: bool,          // ack flag
    pub psh: bool,          // push flag
    pub rst: bool,          // rst flag
    pub syn: bool,          // syn flag
    pub fin: bool,          // fin flag
    pub win: u16,           // window size
    pub cksum: u16,         // checksum
    pub uptr: u16,          // urgent pointer
}

// impl TCPHeader {
//     pub const LENGTH: usize = 20;

//     pub fn parse(buffer: &[u8]) -> Result<Self, &'static str> {
//         if buffer.len() < Self::LENGTH {
//             return Err("Header too short");
//         }

//         let (
//             src_port,
//             dst_port,
//             seq_num,
//             ack_num,
//             data_offset,
//             flags,
//             window_size,
//             checksum,
//             urgent_ptr,
//         ) = {
//             let mut parser = &buffer[..Self::LENGTH];

//             (
//                 parser.read_u16::<BigEndian>().ok_or("Invalid src port")?,
//                 parser.read_u16::<BigEndian>().ok_or("Invalid dst port")?,
//                 parser.read_u32::<BigEndian>().ok_or("Invalid seq num")?,
//                 parser.read_u32::<BigEndian>().ok_or("Invalid ack num")?,
//                 parser.read_u8().ok_or("Invalid data offset")? >> 4,
//                 parser.read_u8().ok_or("Invalid flags")?,
//                 parser
//                     .read_u16::<BigEndian>()
//                     .ok_or("Invalid window size")?,
//                 parser.read_u16::<BigEndian>().ok_or("Invalid checksum")?,
//                 parser.read_u16::<BigEndian>().ok_or("Invalid urgent ptr")?,
//             )
//         };

//         if data_offset < 5 {
//             return Err("Data offset too short");
//         }

//         Ok(Self {
//             src_port,
//             dst_port,
//             seq_num,
//             ack_num,
//             data_offset,
//             urg: flags & 0b0010_0000 != 0,
//             ack: flags & 0b0001_0000 != 0,
//             psh: flags & 0b0000_1000 != 0,
//             rst: flags & 0b0000_0100 != 0,
//             syn: flags & 0b0000_0010 != 0,
//             fin: flags & 0b0000_0001 != 0,
//             window_size,
//             checksum,
//             urgent_ptr,
//         })
//     }

//     // More methods implementing serialize, checksum, etc.
// }
