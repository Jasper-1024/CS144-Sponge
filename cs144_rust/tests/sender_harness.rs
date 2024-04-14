use std::{cell::RefCell, collections::VecDeque, ops::Not, rc::Rc};

use cs144_rust::{
    byte_stream::ByteStreamTrait,
    tcp_helpers::{
        tcp_config::{TCPConfig, MAX_PAYLOAD_SIZE, MAX_RETX_ATTEMPTS},
        tcp_header::TCPHeaderTrait,
        tcp_segment::TCPSegment,
        tcp_state::{TCPSenderStateSummary, TCPState},
    },
    tcp_sender::{TCPSender, TCPSenderTrait},
    wrapping_integers::WrappingInt32,
};

const DEFAULT_TEST_WINDOW: u16 = 137;

/// like SenderTestStep in cpp code
pub trait SenderTestStep {
    fn to_string(&self) -> String {
        String::from("SenderTestStep")
    }
    fn execute(&self, sender: Rc<RefCell<TCPSender>>, segments: &mut VecDeque<TCPSegment>) {
        todo!()
    }
}

/// ExpectState
pub struct ExpectState {
    state: TCPSenderStateSummary,
}

impl ExpectState {
    pub fn new(state: TCPSenderStateSummary) -> Self {
        Self { state }
    }
}

impl SenderTestStep for ExpectState {
    fn to_string(&self) -> String {
        format!("ExpectState({:?})", self.state)
    }

    fn execute(&self, sender: Rc<RefCell<TCPSender>>, _segments: &mut VecDeque<TCPSegment>) {
        let sender = sender.borrow_mut();
        assert_eq!(
            TCPState::state_summary_sender(&*sender),
            self.state,
            "The TCPSender was in state `{}`, but it was expected to be in state `{}`",
            TCPState::state_summary_sender(&*sender),
            self.state
        );
    }
}

pub struct ExpectSeqno {
    seqno: WrappingInt32,
}

impl ExpectSeqno {
    pub fn new(seqno: WrappingInt32) -> Self {
        Self { seqno }
    }
}

impl SenderTestStep for ExpectSeqno {
    fn to_string(&self) -> String {
        format!("ExpectSeqno({})", self.seqno)
    }

    fn execute(&self, sender: Rc<RefCell<TCPSender>>, _segments: &mut VecDeque<TCPSegment>) {
        let sender = sender.borrow();
        assert_eq!(
            sender.next_seqno(),
            self.seqno,
            "The TCPSender reported that the next seqno is {}, but it was expected to be {}",
            sender.next_seqno(),
            self.seqno
        );
    }
}

pub struct ExpectBytesInFlight {
    n_bytes: usize,
}

impl ExpectBytesInFlight {
    pub fn new(n_bytes: usize) -> Self {
        Self { n_bytes }
    }
}

impl SenderTestStep for ExpectBytesInFlight {
    fn to_string(&self) -> String {
        format!("ExpectBytesInFlight({})", self.n_bytes)
    }

    fn execute(&self, sender: Rc<RefCell<TCPSender>>, _segments: &mut VecDeque<TCPSegment>) {
        let sender = sender.borrow();
        assert_eq!(
            sender.bytes_in_flight(),
            self.n_bytes,
            "The TCPSender reported {} bytes in flight, but there was expected to be {} bytes in flight",
            sender.bytes_in_flight(),
            self.n_bytes
        );
    }
}

pub struct ExpectNoSegment {}

impl ExpectNoSegment {
    pub fn new(_: bool) -> Self {
        Self {}
    }
}

impl SenderTestStep for ExpectNoSegment {
    fn to_string(&self) -> String {
        String::from("ExpectNoSegment")
    }

    fn execute(&self, _sender: Rc<RefCell<TCPSender>>, segments: &mut VecDeque<TCPSegment>) {
        assert!(
            segments.is_empty(),
            "The TCPSender sent a segment, but should not have. Segment info:{}",
            segments.back().unwrap().header.summary()
        );
    }
}

pub struct WriteBytes {
    bytes: Vec<u8>,
    end_input: bool,
}

impl WriteBytes {
    pub fn new(bytes: Vec<u8>, end_input: bool) -> Self {
        Self { bytes, end_input }
    }

    pub fn with_end_input(self, end_input: bool) -> Self {
        Self { end_input, ..self }
    }
}

impl SenderTestStep for WriteBytes {
    fn to_string(&self) -> String {
        format!("WriteBytes({:?}, {})", self.bytes, self.end_input)
    }

    fn execute(&self, sender: Rc<RefCell<TCPSender>>, _segments: &mut VecDeque<TCPSegment>) {
        let mut sender = sender.borrow_mut();
        sender.stream_in().borrow_mut().write(self.bytes.as_slice());

        if self.end_input {
            sender.stream_in().borrow_mut().end_input();
        }

        sender.fill_window();
    }
}

pub struct Tick {
    ms: u64,
    max_retx_exceeded: Option<bool>,
}

impl Tick {
    pub fn new(ms: u64) -> Self {
        Self {
            ms,
            max_retx_exceeded: None,
        }
    }

    pub fn with_max_retx_exceeded(self, max_retx_exceeded: bool) -> Self {
        Self {
            max_retx_exceeded: Some(max_retx_exceeded),
            ..self
        }
    }
}

impl SenderTestStep for Tick {
    fn to_string(&self) -> String {
        format!("Tick({})", self.ms)
    }
    fn execute(&self, sender: Rc<RefCell<TCPSender>>, segments: &mut VecDeque<TCPSegment>) {
        let mut sender = sender.borrow_mut();
        sender.tick(self.ms);
        if let Some(max_retx_exceeded) = self.max_retx_exceeded {
            assert_eq!(
                (sender.consecutive_retransmissions() > MAX_RETX_ATTEMPTS),
                max_retx_exceeded,
                "after {} ms passed the TCP Sender reported\n\tconsecutive_retransmissions = {} TCPSender's max_retx_exceeded() was {}, \nbut it should have been\n\t",self.ms, if max_retx_exceeded {
                    "greater than"
                } else {
                    "less than or equal to"
                },MAX_RETX_ATTEMPTS
            );
        }
    }
}

pub struct AckReceived {
    ackno: WrappingInt32,
    window_advertisement: Option<u16>,
}

impl AckReceived {
    pub fn new(ackno: WrappingInt32) -> Self {
        Self {
            ackno,
            window_advertisement: None,
        }
    }

    pub fn with_win(self, win: u16) -> Self {
        Self {
            window_advertisement: Some(win),
            ..self
        }
    }
}

impl SenderTestStep for AckReceived {
    fn to_string(&self) -> String {
        format!(
            "AckReceived({}{}",
            self.ackno,
            if let Some(win) = self.window_advertisement {
                format!(", win: {}", win)
            } else {
                String::new()
            }
        )
    }

    fn execute(&self, sender: Rc<RefCell<TCPSender>>, _segments: &mut VecDeque<TCPSegment>) {
        let mut sender = sender.borrow_mut();
        sender.ack_received(
            self.ackno,
            self.window_advertisement.unwrap_or(DEFAULT_TEST_WINDOW),
        );
        sender.fill_window();
    }
}

pub struct Close;

impl Close {
    pub fn new(_: bool) -> Self {
        Self {}
    }
}

impl SenderTestStep for Close {
    fn to_string(&self) -> String {
        String::from("Close")
    }

    fn execute(&self, sender: Rc<RefCell<TCPSender>>, segments: &mut VecDeque<TCPSegment>) {
        let mut sender = sender.borrow_mut();
        sender.stream_in().borrow_mut().end_input();
        sender.fill_window();
    }
}

// SegmentArrives
#[derive(Debug, PartialEq, Clone)] // Add this line
pub enum Result {
    NotSyn,
    Ok,
}

impl Result {
    fn name(&self) -> &'static str {
        match self {
            Result::NotSyn => "(no SYN received, so no ackno available)",
            Result::Ok => "(SYN received, so ackno available)",
        }
    }
}

// struct SegmentArrivesBuilder {
//     ack: bool,
//     rst: bool,
//     syn: bool,
//     fin: bool,
//     seqno: WrappingInt32,
//     ackno: WrappingInt32,
//     win: u16,
//     payload_size: usize,
//     data: Vec<u8>,
//     result: Option<Result>,
// }

// impl SegmentArrivesBuilder {
//     fn new() -> Self {

//     }
// }

pub struct ExpectSegment {
    ack: Option<bool>,
    rst: Option<bool>,
    syn: Option<bool>,
    fin: Option<bool>,
    seqno: Option<WrappingInt32>,
    ackno: Option<WrappingInt32>,
    win: Option<u16>,
    payload_size: Option<usize>,
    data: Option<Vec<u8>>,
    result: Option<Result>,
}

#[allow(dead_code)]
impl ExpectSegment {
    pub fn new() -> Self {
        Self {
            ack: None,
            rst: None,
            syn: None,
            fin: None,
            seqno: None,
            ackno: None,
            win: None,
            payload_size: None,
            data: None,
            result: None,
        }
    }

    pub fn ack(mut self, ack: bool) -> Self {
        self.ack = Some(ack);
        self
    }
    pub fn rst(mut self, rst: bool) -> Self {
        self.rst = Some(rst);
        self
    }
    pub fn syn(mut self, syn: bool) -> Self {
        self.syn = Some(syn);
        self
    }
    pub fn fin(mut self, fin: bool) -> Self {
        self.fin = Some(fin);
        self
    }
    pub fn no_flags(mut self) -> Self {
        self.ack = Some(false);
        self.rst = Some(false);
        self.syn = Some(false);
        self.fin = Some(false);
        self
    }
    pub fn seqno(mut self, seqno: u32) -> Self {
        self.seqno = Some(WrappingInt32::new(seqno));
        self
    }
    pub fn seqno_warp(mut self, seqno: WrappingInt32) -> Self {
        self.seqno = Some(seqno);
        self
    }
    pub fn ackno(mut self, ackno: u32) -> Self {
        self.ackno = Some(WrappingInt32::new(ackno));
        self
    }
    pub fn ackno_warp(mut self, ackno: WrappingInt32) -> Self {
        self.ackno = Some(ackno);
        self
    }

    pub fn win(mut self, win: u16) -> Self {
        self.win = Some(win);
        self
    }
    pub fn payload_size(mut self, payload_size: usize) -> Self {
        self.payload_size = Some(payload_size);
        self
    }
    pub fn data(mut self, data: Vec<u8>) -> Self {
        self.data = Some(data);
        self
    }
}

macro_rules! assert_eq_flags {
    ($actual:expr, $expected:expr, $flag:literal) => {
        if let Some(expected) = $expected {
            assert!(
                expected == $actual,
                "The segment's {} flag was {}, but it was expected to be {}",
                $flag,
                $actual,
                expected
            );
        }
    };
}

impl SenderTestStep for ExpectSegment {
    fn to_string(&self) -> String {
        format!(
            "SegmentArrives {{ ack: {:?}, rst: {:?}, syn: {:?}, fin: {:?}, seqno: {:?}, ackno: {:?}, win: {:?}, payload_size: {}, data: {:?}, result: {:?} }}",
            self.ack,
            self.rst,
            self.syn,
            self.fin,
            self.seqno,
            self.ackno,
            self.win,
            self.payload_size.unwrap_or_default(),
            self.data,
            self.result
        )
    }

    fn execute(&self, sender: Rc<RefCell<TCPSender>>, segments: &mut VecDeque<TCPSegment>) {
        assert!(
            !segments.is_empty(),
            "The Sender should have produced a segment that existed, but it did not",
        );

        let seg = segments.pop_back();
        assert!(
            seg.is_some(),
            "The seg should have produced a segment that existed, but it did not",
        );
        let seg = seg.unwrap();

        assert_eq_flags!(seg.header.ack, self.ack, "ack");
        assert_eq_flags!(seg.header.rst, self.rst, "rst");
        assert_eq_flags!(seg.header.syn, self.syn, "syn");
        assert_eq_flags!(seg.header.fin, self.fin, "fin");
        assert_eq_flags!(seg.header.seqno, self.seqno, "seqno");
        assert_eq_flags!(seg.header.ackno, self.ackno, "ackno");
        assert_eq_flags!(seg.header.win, self.win, "win");
        assert_eq_flags!(seg.payload.len(), self.payload_size, "payload_size");
        assert!(
            seg.payload.len() <= MAX_PAYLOAD_SIZE,
            "packet has length {} which is greater than the maximum payload size of {}",
            seg.payload.len(),
            MAX_PAYLOAD_SIZE
        );
        if let Some(data) = &self.data {
            assert_eq!(
                seg.payload.as_slice(),
                data,
                "payloads differ. expected {:?}, but found {:?}",
                seg.payload.as_ref(),
                data.as_slice()
            );
        }
    }
}

// The TCPSenderTestHarness struct that holds a reference to TCPSender and executes the test steps.
pub struct TCPSenderTestHarness {
    sender: Rc<RefCell<TCPSender>>,
    outbound_segments: VecDeque<TCPSegment>,
    steps_executed: Vec<String>,
    name: &'static str,
}

impl TCPSenderTestHarness {
    pub fn new(name: &'static str, config: TCPConfig) -> Self {
        let mut sender = TCPSender::new(
            config.send_capacity,
            config.rt_timeout as u64,
            config.fixed_isn,
        );
        sender.fill_window();
        let mut harness = Self {
            sender: Rc::new(RefCell::new(sender)),
            outbound_segments: VecDeque::new(),
            steps_executed: vec![],
            name,
        };
        harness.collect_output();
        harness
    }

    pub fn execute(&mut self, step: &dyn SenderTestStep) {
        step.execute(self.sender.clone(), &mut self.outbound_segments);
        self.collect_output();
    }

    fn collect_output(&mut self) {
        // Collect segments from the sender's output and push them to outbound_segments.
        let mut sender = self.sender.borrow_mut();
        while !sender.segments_out.is_empty() {
            self.outbound_segments
                .push_back(sender.segments_out.pop_front().unwrap());
        }
    }
}

// #[macro_export]
// macro_rules! execute_test {
//     ($test:expr, $step:ident, $value:expr) => {
//         let step = $step::new($value);
//         $test.execute(&step);
//     };
// }

#[macro_export]
macro_rules! execute_test {
    ($test:expr, $step:ident, $($value:expr),*) => {
        let step = $step::new($($value),*);
        $test.execute(&step);
    };
}
