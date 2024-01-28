use super::tcp_header::TCPHeader;
use crate::util::buffer::Buffer;

pub(crate) struct TCPSegment {
    pub(crate) header: TCPHeader,
    pub(crate) payload: Buffer, // a warp of arc<String>
}

impl TCPSegment {
    pub fn new(header: TCPHeader, payload: Buffer) -> Self {
        TCPSegment { header, payload }
    }
}
