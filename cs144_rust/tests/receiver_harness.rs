use std::{cell::RefCell, rc::Rc, u8, usize};

use cs144_rust::{
    byte_stream::ByteStreamTrait,
    tcp_helpers::{
        tcp_header::TCPHeaderTrait,
        tcp_segment::TCPSegment,
        tcp_state::{TCPReceiverStateSummary, TCPState},
    },
    tcp_receiver::{TCPReceiver, TCPReceiverTrait},
    wrapping_integers::WrappingInt32,
};

// 对应于 C++ 中的 ReceiverTestStep 结构体
// 任何测试步骤的 trait
pub trait ReceiverTestStep {
    fn to_string(&self) -> String {
        String::from("ReceiverTestStep")
    }
    fn execute(&self, receiver: Rc<RefCell<TCPReceiver>>) {
        todo!()
    }
    fn execute_with_segment<'a>(
        &self,
        receiver: Rc<RefCell<TCPReceiver<'a>>>,
        seg: &'a TCPSegment,
    ) {
        todo!()
    }
}

// 对应于 C++ 中的 ReceiverExpectationViolation 类
// #[derive(Debug)]
// pub struct ReceiverExpectationViolation {
//     msg: String,
// }

// impl ReceiverExpectationViolation {
//     pub fn new(msg: String) -> Self {
//         Self { msg }
//     }
// }

// impl std::fmt::Display for ReceiverExpectationViolation {
//     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//         write!(f, "{}", self.msg)
//     }
// }

// impl std::error::Error for ReceiverExpectationViolation {}

// 对应于 C++ 中的 ReceiverExpectation 结构体
// pub trait ReceiverExpectation: for<'a> ReceiverTestStep<'a> {
//     fn description(&self) -> String {
//         String::from("description missing")
//     }
//     fn to_string(&self) -> String {
//         format!("Expectation: {}", self.description())
//     }
// }

// 开始真正的测试步骤
// 对应于 C++ 中的 ExpectState 结构体
pub struct ExpectState {
    state: TCPReceiverStateSummary,
}

impl ExpectState {
    pub fn new(state: TCPReceiverStateSummary) -> Self {
        Self { state }
    }
}

// impl ReceiverExpectation for ExpectState {
//     fn description(&self) -> String {
//         format!("in state `{}`", self.state)
//     }

//     fn to_string(&self) -> String {
//         format!("Expectation: {}", self.description())
//     }
// }

impl ReceiverTestStep for ExpectState {
    fn execute(&self, receiver: Rc<RefCell<TCPReceiver>>) {
        let receiver = receiver.borrow_mut();
        // 解析状态与预期状态是否一致
        assert_eq!(
            TCPState::state_summary_receiver(&*receiver),
            self.state,
            "The TCPReceiver was in state `{}`, but it was expected to be in state `{}`",
            TCPState::state_summary_receiver(&*receiver),
            self.state
        );
    }
}

// ExpectAckno
pub struct ExpectAckno {
    ackno: Option<WrappingInt32>,
}

impl ExpectAckno {
    pub fn new(ackno: Option<u32>) -> Self {
        match ackno {
            Some(ackno) => Self {
                ackno: Some(WrappingInt32::new(ackno)),
            },
            None => Self { ackno: None },
        }
    }
}

impl ReceiverTestStep for ExpectAckno {
    fn execute(&self, receiver: Rc<RefCell<TCPReceiver>>) {
        let receiver = receiver.borrow();
        assert_eq!(
            receiver.ackno(),
            self.ackno,
            "The TCPReceiver reported ackno `{}`, but it was expected to be `{}`",
            receiver
                .ackno()
                .map_or("None".to_string(), |x| x.to_string()),
            self.ackno.map_or("None".to_string(), |x| x.to_string())
        );
    }
}
// ExpectWindow
pub struct ExpectWindow {
    window: usize,
}

impl ExpectWindow {
    pub fn new(window: usize) -> Self {
        Self { window }
    }
}

impl ReceiverTestStep for ExpectWindow {
    fn execute(&self, receiver: Rc<RefCell<TCPReceiver>>) {
        let receiver = receiver.borrow();
        assert_eq!(
            receiver.window_size(),
            self.window,
            "The TCPReceiver reported window size `{}`, but it was expected to be `{}`",
            receiver.window_size(),
            self.window
        );
    }
}

// ExpectUnassembledBytes
pub struct ExpectUnassembledBytes {
    n_bytes: usize,
}
impl ExpectUnassembledBytes {
    pub fn new(n_bytes: usize) -> Self {
        Self { n_bytes }
    }
}

impl ReceiverTestStep for ExpectUnassembledBytes {
    fn execute(&self, receiver: Rc<RefCell<TCPReceiver>>) {
        assert_eq!(
            receiver.borrow().unassembled_bytes(),
            self.n_bytes,
            "The TCPReceiver reported {} unassembled bytes, but it was expected to be {}",
            receiver.borrow().unassembled_bytes(),
            self.n_bytes
        );
    }
}

// ExpectTotalAssembledBytes
pub struct ExpectTotalAssembledBytes {
    n_bytes: usize,
}

impl ExpectTotalAssembledBytes {
    pub fn new(n_bytes: usize) -> Self {
        Self { n_bytes }
    }
}

impl ReceiverTestStep for ExpectTotalAssembledBytes {
    fn execute(&self, receiver: Rc<RefCell<TCPReceiver>>) {
        assert_eq!(
            receiver.borrow().stream_out().borrow().bytes_written(),
            self.n_bytes as u64,
            "The TCPReceiver reported {} total assembled bytes, but it was expected to be {}",
            receiver.borrow().stream_out().borrow().bytes_written(),
            self.n_bytes
        );
    }
}

// ExpectEof | receiver.stream_out().eof() == true
pub struct ExpectEof {
    flag: bool,
}

impl ExpectEof {
    pub fn new(flag: bool) -> Self {
        Self { flag }
    }
}

impl ReceiverTestStep for ExpectEof {
    fn execute(&self, receiver: Rc<RefCell<TCPReceiver>>) {
        assert!(
            receiver.borrow().stream_out().borrow().eof(),
            "The TCPReceiver reported that the output stream was not at EOF, but it was expected to be at EOF"
        );
    }
}

// ExpectInputNotEnded | receiver.stream_out().input_ended() == false
pub struct ExpectInputNotEnded {
    flag: bool,
}

impl ExpectInputNotEnded {
    pub fn new(flag: bool) -> Self {
        Self { flag }
    }
}

impl ReceiverTestStep for ExpectInputNotEnded {
    fn execute(&self, receiver: Rc<RefCell<TCPReceiver>>) {
        assert!(
            !receiver.borrow().stream_out().borrow().input_ended(),
            "The TCPReceiver reported that the input stream was at EOF, but it was expected to not be at EOF"
        );
    }
}

// ExpectBytes
pub struct ExpectBytes {
    bytes: Vec<u8>,
}

impl ExpectBytes {
    pub fn new(data: Vec<u8>) -> Self {
        Self { bytes: data }
    }
}

impl ReceiverTestStep for ExpectBytes {
    fn execute(&self, receiver: Rc<RefCell<TCPReceiver>>) {
        let binding = receiver.borrow().stream_out();

        let res = binding.borrow().buffer_size();
        assert_eq!(
            res, self.bytes.len(),
            "The TCPReceiver reported that the output stream had a buffer size of {}, but it was expected to be {}",
            res,
            self.bytes.len()
        );
        let bytes = binding.borrow_mut().read(self.bytes.len());
        assert_eq!(
            bytes, self.bytes,
            "The TCPReceiver reported that the output stream had the bytes {:?}, but it was expected to be {:?}",
            bytes,
            self.bytes
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

pub struct SegmentArrives {
    pub ack: bool,
    pub rst: bool,
    pub syn: bool,
    pub fin: bool,
    pub seqno: WrappingInt32,
    pub ackno: WrappingInt32,
    pub win: u16,
    pub data: Vec<u8>,
    pub result: Option<Result>,
}

impl SegmentArrives {
    pub fn default() -> Self {
        Self {
            ack: false,
            rst: false,
            syn: false,
            fin: false,
            seqno: WrappingInt32::new(0),
            ackno: WrappingInt32::new(0),
            win: 0,
            data: vec![],
            result: None,
        }
    }
    // build_segment
    pub fn build_segment(&mut self) -> TCPSegment {
        let mut seg = TCPSegment::default();
        seg.payload = self.data.clone().into();
        seg.header.ack = self.ack;
        seg.header.fin = self.fin;
        seg.header.syn = self.syn;
        seg.header.rst = self.rst;
        seg.header.ackno = self.ackno;
        seg.header.seqno = self.seqno;
        seg.header.win = self.win;
        seg
    }
}

pub struct SegmentArrivesBuilder {
    ack: bool,
    rst: bool,
    syn: bool,
    fin: bool,
    seqno: WrappingInt32,
    ackno: WrappingInt32,
    win: u16,
    data: Vec<u8>,
    result: Option<Result>,
}

impl SegmentArrivesBuilder {
    pub fn new() -> Self {
        Self {
            ack: false,
            rst: false,
            syn: false,
            fin: false,
            seqno: WrappingInt32::new(0),
            ackno: WrappingInt32::new(0),
            win: 0,
            data: vec![],
            result: None,
        }
    }

    pub fn ack(mut self, ackno: u32) -> Self {
        self.ack = true;
        self.ackno = WrappingInt32::new(ackno);
        self
    }

    pub fn rst(mut self) -> Self {
        self.rst = true;
        self
    }

    pub fn syn(mut self) -> Self {
        self.syn = true;
        self
    }

    pub fn fin(mut self) -> Self {
        self.fin = true;
        self
    }

    pub fn seqno(mut self, seqno: u32) -> Self {
        self.seqno = seqno.into();
        self
    }

    pub fn ackno(mut self, ackno: u32) -> Self {
        self.ackno = ackno.into();
        self
    }

    pub fn win(mut self, win: u16) -> Self {
        self.win = win;
        self
    }

    pub fn data<const N: usize>(mut self, data: [u8; N]) -> Self {
        self.data = data.to_vec();
        self
    }

    pub fn data_vec(mut self, data: Vec<u8>) -> Self {
        self.data = data;
        self
    }

    pub fn result(mut self, result: Option<Result>) -> Self {
        self.result = result;
        self
    }

    pub fn build<'a>(self) -> SegmentArrives {
        let mut seg_arrivers = SegmentArrives::default();
        seg_arrivers.ack = self.ack;
        seg_arrivers.fin = self.fin;
        seg_arrivers.syn = self.syn;
        seg_arrivers.rst = self.rst;
        seg_arrivers.ackno = self.ackno;
        seg_arrivers.seqno = self.seqno;
        seg_arrivers.win = self.win;
        seg_arrivers.data = self.data;
        seg_arrivers.result = self.result;

        seg_arrivers
    }
}

impl ReceiverTestStep for SegmentArrives {
    // just for None
    // fn execute(&self, receiver: Rc<RefCell<TCPReceiver>>) {
    //     let seg = self.build_segment();
    //     let mut o = String::new();
    //     o.push_str(&seg.header.summary());
    //     if !self.data.is_empty() {
    //         o.push_str(&format!(" with data {:?}", self.data));
    //     }

    //     receiver.borrow_mut().segment_received(&seg);

    //     let res = if receiver.borrow().ackno().is_none() {
    //         Result::NotSyn
    //     } else {
    //         Result::Ok
    //     };

    //     if let Some(expected_res) = self.result.clone() {
    //         assert_eq!(
    //             res,
    //             expected_res,
    //             "TCPReceiver::segment_received() reported `{}` in response to `{}`, but it was expected to report `{}`",
    //             res.name(),
    //             o,
    //             expected_res.name()
    //         );
    //     }
    // }

    fn execute_with_segment<'a>(
        &self,
        receiver: Rc<RefCell<TCPReceiver<'a>>>,
        seg: &'a TCPSegment,
    ) {
        // let seg = self.build_segment();
        let mut o = String::new();
        o.push_str(&seg.header.summary());
        if !self.data.is_empty() {
            o.push_str(&format!(" with data {:?}", self.data));
        }

        receiver.borrow_mut().segment_received(seg);

        let res = if receiver.borrow().ackno().is_none() {
            Result::NotSyn
        } else {
            Result::Ok
        };

        if let Some(expected_res) = self.result.clone() {
            assert_eq!(
                res,
                expected_res,
                "TCPReceiver::segment_received() reported `{}` in response to `{}`, but it was expected to report `{}`",
                res.name(),
                o,
                expected_res.name()
            );
        }
    }
}

// 对应于 C++ 中的 TCPReceiverTestHarness 类
// 持有 TCPReceiver 实例, 执行 任何实现了 ReceiverTestStep 的测试步骤
pub struct TCPReceiverTestHarness<'a> {
    receiver: Rc<RefCell<TCPReceiver<'a>>>,
}

impl<'a> TCPReceiverTestHarness<'a> {
    // 创建一个新的 TCPReceiverTestHarness
    pub fn new(capacity: usize) -> Self {
        let receiver = TCPReceiver::new(capacity);
        Self {
            receiver: Rc::new(RefCell::new(receiver)),
        }
    }

    // 执行所有的测试步骤
    pub fn execute(&self, step: &'a dyn ReceiverTestStep) {
        step.execute(self.receiver.clone());
    }

    pub fn execute_with_segment(&self, step: &'a dyn ReceiverTestStep, seg: &'a TCPSegment) {
        step.execute_with_segment(self.receiver.clone(), seg);
    }
}

#[macro_export]
macro_rules! execute_test {
    ($test:expr, $step:ident, $value:expr) => {
        let step = $step::new($value);
        $test.execute(&step);
    };
}
