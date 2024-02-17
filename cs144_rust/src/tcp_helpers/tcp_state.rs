use core::fmt;

use crate::{
    byte_stream::ByteStreamTrait,
    tcp_receiver::{TCPReceiver, TCPReceiverTrait},
};

#[derive(Debug, PartialEq)]
pub enum TCPReceiverStateSummary {
    Error,
    Listen,
    SynReceived,
    FinReceived,
}

impl fmt::Display for TCPReceiverStateSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TCPReceiverStateSummary::Error => write!(f, "error (connection was reset)"),
            TCPReceiverStateSummary::Listen => write!(f, "waiting for SYN: ackno is empty"),
            TCPReceiverStateSummary::SynReceived => {
                write!(
                    f,
                    "SYN received (ackno exists), and input to stream hasn't ended"
                )
            }
            TCPReceiverStateSummary::FinReceived => write!(f, "input to stream has ended"),
        }
    }
}

pub struct TCPState {}

impl TCPState {
    pub fn state_summary(receiver: &TCPReceiver) -> TCPReceiverStateSummary {
        if receiver.stream_out().borrow().error() {
            return TCPReceiverStateSummary::Error;
        }
        if receiver.ackno().is_none() {
            return TCPReceiverStateSummary::Listen;
        }
        if receiver.stream_out().borrow().input_ended() {
            return TCPReceiverStateSummary::FinReceived;
        }
        return TCPReceiverStateSummary::SynReceived;
    }
}
