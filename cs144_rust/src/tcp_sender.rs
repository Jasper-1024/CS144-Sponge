use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use crate::{
    byte_stream::ByteStream, tcp_helpers::tcp_segment::TCPSegment, wrapping_integers::WrappingInt32,
};

pub struct TCPSender {
    isn: WrappingInt32,
    segments_out: VecDeque<TCPSegment>,
    initial_retransmission_timeout: u64,
    stream: Rc<RefCell<ByteStream>>,
    next_seqno: u64,
}

impl TCPSender {}

pub trait TCPSenderTrait {
    fn fill_window(&mut self);
    fn ack_received(&mut self, ackno: WrappingInt32, window_size: u16);
    fn tick(&mut self, ms_since_last_tick: u64);
    fn send_empty_segment(&mut self);
    // other functions
    /// \brief How many sequence numbers are occupied by segments sent but not yet acknowledged?
    /// \note count is in "sequence space," i.e. SYN and FIN each count for one byte
    /// (see TCPSegment::length_in_sequence_space())
    fn bytes_in_flight(&self) -> usize;

    /// \brief Number of consecutive retransmissions that have occurred in a row
    fn consecutive_retransmissions(&self) -> usize;

    fn segments_out(&self) -> &VecDeque<TCPSegment>;

    /// \name What is the next sequence number? (used for testing)
    /// \brief absolute seqno for the next byte to be sent
    fn next_seqno_absolute(&self) -> u64;

    /// \brief relative seqno for the next byte to be sent
    fn next_seqno(&self) -> WrappingInt32;

    fn stream_in(&self) -> Rc<RefCell<ByteStream>>;
}

impl TCPSenderTrait for TCPSender {
    fn fill_window(&mut self) {
        unimplemented!()
    }

    fn ack_received(&mut self, ackno: WrappingInt32, window_size: u16) {
        unimplemented!()
    }

    fn tick(&mut self, ms_since_last_tick: u64) {
        unimplemented!()
    }

    fn send_empty_segment(&mut self) {
        unimplemented!()
    }

    fn bytes_in_flight(&self) -> usize {
        unimplemented!()
    }

    fn consecutive_retransmissions(&self) -> usize {
        unimplemented!()
    }

    fn segments_out(&self) -> &VecDeque<TCPSegment> {
        &self.segments_out
    }

    fn next_seqno_absolute(&self) -> u64 {
        self.next_seqno
    }

    fn next_seqno(&self) -> WrappingInt32 {
        WrappingInt32::wrap(self.next_seqno, self.isn)
    }

    fn stream_in(&self) -> Rc<RefCell<ByteStream>> {
        unimplemented!()
    }
}
