use std::{cell::RefCell, collections::VecDeque, ops::Not, rc::Rc};

use cs144_rust::{
    tcp_helpers::{
        tcp_config::{TCPConfig, MAX_PAYLOAD_SIZE},
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

pub struct SegmentArrives {
    ack: bool,
    rst: bool,
    syn: bool,
    fin: bool,
    seqno: WrappingInt32,
    ackno: WrappingInt32,
    win: u16,
    payload_size: usize,
    data: Vec<u8>,
    result: Option<Result>,
}

#[allow(dead_code)]
impl SegmentArrives {
    pub fn new() -> Self {
        Self {
            ack: false,
            rst: false,
            syn: false,
            fin: false,
            seqno: WrappingInt32::new(0),
            ackno: WrappingInt32::new(0),
            win: 0,
            payload_size: 0,
            data: vec![],
            result: Some(Result::NotSyn),
        }
    }
    pub fn no_flags(mut self) -> Self {
        self.ack = false;
        self.rst = false;
        self.syn = false;
        self.fin = false;
        self
    }
    pub fn ack(mut self, ackno: u32) -> Self {
        self.ack = true;
        self.ackno = WrappingInt32::new(ackno);
        self
    }
    pub fn rst(mut self, rst: bool) -> Self {
        self.rst = rst;
        self
    }
    pub fn syn(mut self, syn: bool) -> Self {
        self.syn = syn;
        self
    }
    pub fn fin(mut self, fin: bool) -> Self {
        self.fin = fin;
        self
    }
    pub fn seqno(mut self, seqno: u32) -> Self {
        self.seqno = WrappingInt32::new(seqno);
        self
    }
    pub fn win(mut self, win: u16) -> Self {
        self.win = win;
        self
    }
    pub fn payload_size(mut self, payload_size: usize) -> Self {
        self.payload_size = payload_size;
        self
    }
    pub fn data(mut self, data: Vec<u8>) -> Self {
        self.data = data;
        self
    }
}

macro_rules! assert_eq_flags {
    ($actual:expr, $expected:expr, $flag:literal) => {
        assert_eq!(
            $actual, $expected,
            "The segment's {} flag was {}, but it was expected to be {}",
            $flag, $actual, $expected
        );
    };
}

impl SenderTestStep for SegmentArrives {
    fn to_string(&self) -> String {
        format!(
            "SegmentArrives {{ ack: {}, rst: {}, syn: {}, fin: {}, seqno: {}, ackno: {}, win: {}, payload_size: {}, data: {:?}, result: {:?} }}",
            self.ack,
            self.rst,
            self.syn,
            self.fin,
            self.seqno,
            self.ackno,
            self.win,
            self.payload_size,
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
            seg.payload.len() < MAX_PAYLOAD_SIZE,
            "packet has length {} which is greater than the maximum payload size of {}",
            seg.payload.len(),
            MAX_PAYLOAD_SIZE
        );
        assert_eq!(
            seg.payload.as_ref(),
            self.data.as_slice(),
            "payloads differ. expected {:?}, but found {:?}",
            seg.payload.as_ref(),
            self.data.as_slice()
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
