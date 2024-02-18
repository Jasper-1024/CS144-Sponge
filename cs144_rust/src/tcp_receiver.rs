use std::{cell::RefCell, rc::Rc};

use crate::{
    byte_stream::{ByteStream, ByteStreamTrait},
    stream_reassembler::{StreamReassembler, StreamReassemblerTrait},
    tcp_helpers::tcp_segment::TCPSegment,
    wrapping_integers::WrappingInt32,
};

pub struct TCPReceiver<'a> {
    reassembler: StreamReassembler<'a>,
    capacity: usize,    // maximum number of bytes that can be stored
    syn_received: bool, // whether SYN has been received
    isn: WrappingInt32, // initial sequence number
}

impl TCPReceiver<'_> {
    pub fn new(capacity: usize) -> Self {
        let reassembler = StreamReassembler::new(capacity);
        TCPReceiver {
            reassembler,
            capacity,
            syn_received: false,
            isn: WrappingInt32::new(0),
        }
    }
}

pub trait TCPReceiverTrait<'a> {
    fn ackno(&self) -> Option<WrappingInt32>; // next ackno
    fn window_size(&self) -> usize; // window size
    fn unassembled_bytes(&self) -> usize; // unassembled bytes
    fn segment_received(&mut self, seg: &'a TCPSegment); // handle segment
    fn stream_out(&self) -> Rc<RefCell<ByteStream>>; // get stream
}

impl<'a> TCPReceiverTrait<'a> for TCPReceiver<'a> {
    fn ackno(&self) -> Option<WrappingInt32> {
        if !self.syn_received {
            // on licensed
            None
        } else {
            // syn
            let mut temp_ackno = self.reassembler.stream_out().borrow().bytes_written() + 1;
            // fin if input_ended() == 1
            temp_ackno += self.reassembler.stream_out().borrow().input_ended() as u64;
            Some(WrappingInt32::wrap(temp_ackno, self.isn))
        }
    }

    fn window_size(&self) -> usize {
        self.capacity - self.reassembler.stream_out().borrow().buffer_size()
    }

    fn unassembled_bytes(&self) -> usize {
        self.reassembler.unassembled_bytes()
    }

    fn segment_received(&mut self, seg: &'a TCPSegment) {
        let header = seg.header;
        if !self.syn_received {
            if !header.syn {
                return;
            }
            self.isn = header.seqno;
            self.syn_received = true;
        }
        // ackno
        let abs_ackno = self.reassembler.stream_out().borrow().bytes_written() + 1;
        let curr_abs_seqno = header.seqno.unwrap(self.isn, abs_ackno);

        let stram_index = curr_abs_seqno - 1 + if header.syn { 1 } else { 0 };

        self.reassembler
            .push_substring(seg.payload.as_ref(), stram_index, header.fin);
    }

    fn stream_out(&self) -> Rc<RefCell<ByteStream>> {
        self.reassembler.stream_out()
    }
}
