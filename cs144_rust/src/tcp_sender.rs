use crate::{byte_stream::*, tcp_helpers::tcp_segment::TCPSegmentTrait, util::buffer::Buffer};
use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use crate::{
    byte_stream::ByteStream,
    tcp_helpers::{tcp_config::MAX_PAYLOAD_SIZE, tcp_segment::TCPSegment},
    wrapping_integers::WrappingInt32,
};

/**
 * 非常简单的计数器封装, 真正的 tick 是外部推动;
 */
pub struct Timer {
    time_count: u64,
    time_out: u64,
    is_running: bool,
}
impl Timer {
    fn new(time_out: u64) -> Self {
        Timer {
            time_count: 0,
            time_out,
            is_running: false,
        }
    }

    fn stop(&mut self) {
        self.is_running = false;
    }
    fn set_time_out(&mut self, time_out: u64) {
        self.time_out = time_out;
    }
    fn get_time_out(&self) -> u64 {
        self.time_out
    }
    fn restart(&mut self) {
        self.time_count = 0;
        self.is_running = true;
    }

    fn tick(&mut self, ms_since_last_tick: u64) {
        if self.is_running {
            self.time_count += ms_since_last_tick;
        }
    }

    fn check(&self) -> bool {
        self.is_running && self.time_count >= self.time_out
    }

    fn is_running(&self) -> bool {
        self.is_running
    }
}

impl Default for Timer {
    fn default() -> Self {
        Timer {
            time_count: 0,
            time_out: 0,
            is_running: false,
        }
    }
}

pub struct TCPSender {
    isn: WrappingInt32,
    pub segments_out: VecDeque<TCPSegment>,
    _timeout: u64,
    stream: Rc<RefCell<ByteStream>>,
    next_seqno: u64,
    // private fields
    window_size: u16,
    bytes_in_flight: usize,
    set_syn: bool,
    set_fin: bool,
    _outstanding_seg: VecDeque<(u64, TCPSegment)>,
    // timer
    _timer: Timer,
    _retrans_count: usize,
}

impl TCPSender {
    pub fn new(capacity: usize, retx_timeout: u64, fixed_isn: Option<WrappingInt32>) -> Self {
        TCPSender {
            isn: fixed_isn.unwrap_or(WrappingInt32::default()), //todo: check default
            segments_out: VecDeque::new(),
            _timeout: retx_timeout,
            stream: Rc::new(RefCell::new(ByteStream::new(capacity))),
            next_seqno: 0,
            window_size: 1,
            bytes_in_flight: 0,
            set_syn: false,
            set_fin: false,
            _outstanding_seg: VecDeque::new(),
            _timer: Timer::new(retx_timeout),
            _retrans_count: 0,
        }
    }
}

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
        let window_size = std::cmp::max(self.window_size, 1); // make sure window_size is at least 1

        while self.bytes_in_flight() < window_size as usize {
            let mut seg = TCPSegment::default();

            if !self.set_syn {
                // first segment, syn without payload(window_size = 1)
                seg.header.syn = true;
                self.set_syn = true;
            }

            let a = window_size as usize
                - self.bytes_in_flight
                - if seg.header.syn { 1 } else { 0 };

            let payload_size = std::cmp::min(
                MAX_PAYLOAD_SIZE,
                std::cmp::min(a, self.stream.borrow().buffer_size()),
            );

            let payload = self.stream.borrow_mut().read(payload_size);
            seg.payload = Buffer::from(payload);

            // meet eof and seg still has space
            if !self.set_fin
                && self.stream.borrow().eof()
                && (self.bytes_in_flight + seg.length_in_sequence_space()
                    < window_size as usize)
            {
                seg.header.fin = true;
                self.set_fin = true;
            }

            let length = seg.length_in_sequence_space();
            if length == 0 {
                break;
            }

            seg.header.seqno = self.next_seqno(); // get the next seqno
            self.segments_out.push_back(seg.clone()); // send the segment

            // if timer is not running, start it
            if !self._timer.is_running() {
                self._timer.restart();
            }

            self._outstanding_seg.push_back((self.next_seqno, seg)); // outstanding queue

            self.next_seqno += length as u64; // absolute seqno update
            self.bytes_in_flight += length; // send but not acked
        }
    }

    fn ack_received(&mut self, ackno: WrappingInt32, window_size: u16) {
        let abs_ackno = ackno.unwrap(self.isn, self.next_seqno_absolute());
        // err ack
        if abs_ackno > self.next_seqno_absolute() {
            return;
        }

        let mut is_work_ack = false;

        // check the outstanding segments
        while !self._outstanding_seg.is_empty() {
            let (abs_seq, seg) = self._outstanding_seg.front().unwrap();
            if abs_seq + seg.length_in_sequence_space() as u64 - 1 < abs_ackno {
                is_work_ack = true;
                self.bytes_in_flight -= seg.length_in_sequence_space();
                self._outstanding_seg.pop_front();
            } else {
                break;
            }
        }
        // 任何这一轮确认的段都会导致计时器重置
        if is_work_ack {
            self._retrans_count = 0; // 连续重传计数 清零
            self._timer.set_time_out(self._timeout); // 重置计时器
            self._timer.restart();
        }
        // 所有发出的段都已确认, 停止计时器
        if self.bytes_in_flight == 0 {
            self._timer.stop();
        }

        self.window_size = window_size; // update window size
        self.fill_window(); // fill the window
    }

    fn tick(&mut self, ms_since_last_tick: u64) {
        self._timer.tick(ms_since_last_tick);
        // 超时
        if self._timer.check() {
            self.segments_out
                .push_back(self._outstanding_seg.front().unwrap().1.clone()); // 重传首个未确认的包

            if self.window_size > 0 {
                self._retrans_count += 1; // 累计重传 +1
                self._timer.set_time_out(self._timer.get_time_out() * 2); // 重传超时时间翻倍
            }
            self._timer.restart(); // 重启计时器
        }
    }

    // only for ack
    fn send_empty_segment(&mut self) {
        let mut seg = TCPSegment::default();
        seg.header.seqno = self.next_seqno();
        self.segments_out.push_back(seg);
    }

    fn bytes_in_flight(&self) -> usize {
        self.bytes_in_flight
    }

    fn consecutive_retransmissions(&self) -> usize {
        self._retrans_count
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
        self.stream.clone()
    }
}
