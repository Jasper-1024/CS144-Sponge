use std::usize;

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
pub trait ReceiverTestStep<'a> {
    fn to_string(&self) -> String {
        String::from("ReceiverTestStep")
    }
    fn execute(&self, receiver: &'a mut TCPReceiver<'a>);
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

impl<'a> ReceiverTestStep<'a> for ExpectState {
    fn execute(&self, receiver: &'a mut TCPReceiver) {
        // 解析状态与预期状态是否一致
        assert_eq!(
            TCPState::state_summary(receiver),
            self.state,
            "The TCPReceiver was in state `{}`, but it was expected to be in state `{}`",
            TCPState::state_summary(receiver),
            self.state
        );
    }
}

// ExpectAckno
pub struct ExpectAckno {
    ackno: Option<WrappingInt32>,
}

impl<'a> ReceiverTestStep<'a> for ExpectAckno {
    fn execute(&self, receiver: &'a mut TCPReceiver) {
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

impl<'a> ReceiverTestStep<'a> for ExpectWindow {
    fn execute(&self, receiver: &'a mut TCPReceiver) {
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
impl<'a> ReceiverTestStep<'a> for ExpectUnassembledBytes {
    fn execute(&self, receiver: &'a mut TCPReceiver) {
        assert_eq!(
            receiver.unassembled_bytes(),
            self.n_bytes,
            "The TCPReceiver reported {} unassembled bytes, but it was expected to be {}",
            receiver.unassembled_bytes(),
            self.n_bytes
        );
    }
}

// ExpectTotalAssembledBytes
pub struct ExpectTotalAssembledBytes {
    n_bytes: usize,
}

impl<'a> ReceiverTestStep<'a> for ExpectTotalAssembledBytes {
    fn execute(&self, receiver: &'a mut TCPReceiver) {
        assert_eq!(
            receiver.stream_out().borrow().bytes_written(),
            self.n_bytes as u64,
            "The TCPReceiver reported {} total assembled bytes, but it was expected to be {}",
            receiver.stream_out().borrow().bytes_written(),
            self.n_bytes
        );
    }
}

// ExpectEof | receiver.stream_out().eof() == true
pub struct ExpectEof;

impl<'a> ReceiverTestStep<'a> for ExpectEof {
    fn execute(&self, receiver: &'a mut TCPReceiver) {
        assert!(
            receiver.stream_out().borrow().eof(),
            "The TCPReceiver reported that the output stream was not at EOF, but it was expected to be at EOF"
        );
    }
}

// ExpectInputNotEnded | receiver.stream_out().input_ended() == false
pub struct ExpectInputNotEnded;

impl<'a> ReceiverTestStep<'a> for ExpectInputNotEnded {
    fn execute(&self, receiver: &'a mut TCPReceiver) {
        assert!(
            !receiver.stream_out().borrow().input_ended(),
            "The TCPReceiver reported that the input stream was at EOF, but it was expected to not be at EOF"
        );
    }
}

// ExpectBytes
pub struct ExpectBytes<const N: usize> {
    bytes: [u8; N],
}

impl<const N: usize> ExpectBytes<N> {
    pub fn new(bytes: [u8; N]) -> Self {
        Self { bytes }
    }
}

impl<'a, const N: usize> ReceiverTestStep<'a> for ExpectBytes<N> {
    fn execute(&self, receiver: &'a mut TCPReceiver) {
        let binding = receiver.stream_out();

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

pub struct SegmentArrives<'a, const N: usize> {
    ack: bool,
    rst: bool,
    syn: bool,
    fin: bool,
    seqno: WrappingInt32,
    ackno: WrappingInt32,
    win: u16,
    data: [u8; N],
    result: Option<Result>,
    tcp_segment: &'a TCPSegment,
}

impl<'a, const N: usize> SegmentArrives<'a, N> {
    // build_segment
    fn build_segment(&self) -> TCPSegment {
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

// impl<'a, const N: usize> SegmentArrives<'a, N> {
//     fn execute(&self, receiver: &'a mut TCPReceiver<'a>) {
//         // let seg = self.build_segment();
//         let mut o = String::new();
//         o.push_str(&self.tcp_segment.header.summary());
//         if !self.data.is_empty() {
//             o.push_str(&format!(" with data {:?}", self.data));
//         }

//         receiver.segment_received(self.tcp_segment);
//     }
// }

impl<'a, const N: usize> ReceiverTestStep<'a> for SegmentArrives<'a, N> {
    fn execute(&self, receiver: &'a mut TCPReceiver<'a>) {
        // let seg = self.build_segment();
        let mut o = String::new();
        o.push_str(&self.tcp_segment.header.summary());
        if !self.data.is_empty() {
            o.push_str(&format!(" with data {:?}", self.data));
        }

        // 这里不得不使用 unsafe 代码, ackno 需要不可变引用,无论如何都无法绕过.
        let receiver_ref: &'a TCPReceiver<'a> = unsafe { std::mem::transmute_copy(&receiver) };

        receiver.segment_received(self.tcp_segment);

        let res = if receiver_ref.ackno().is_none() {
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
    receiver: TCPReceiver<'a>,
}

impl<'a> TCPReceiverTestHarness<'a> {
    // 创建一个新的 TCPReceiverTestHarness
    pub fn new(capacity: usize) -> Self {
        let receiver = TCPReceiver::new(capacity);
        Self { receiver }
    }

    // 执行所有的测试步骤
    pub fn execute(&'a mut self, step: &dyn ReceiverTestStep<'a>) {
        step.execute(&mut self.receiver);
    }
}
