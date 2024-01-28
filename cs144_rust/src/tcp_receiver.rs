use std::{cell::RefCell, rc::Rc};

use crate::{
    byte_stream::ByteStream, stream_reassembler::StreamReassembler, tcp_helpers::tcp_segment::TCPSegment, wrapping_integers::WrappingInt32
};

pub(crate) struct TCPReceiver<'a> {
    reassembler: StreamReassembler<'a>,
    capacity: usize,
}

pub(crate) trait TCPReceiverTrait {
    fn new(capacity: usize) -> Self;
    fn ackno(&self) -> Option<WrappingInt32>;
    fn window_size(&self) -> usize;
    fn unassembled_bytes(&self) -> usize;
    fn segment_received(&mut self, seg: TCPSegment);
    fn stream_out(&mut self) -> Rc<RefCell<ByteStream>>;
}
