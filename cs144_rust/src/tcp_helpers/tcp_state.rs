use core::fmt;

use crate::{
    byte_stream::ByteStreamTrait,
    tcp_receiver::{TCPReceiver, TCPReceiverTrait},
    tcp_sender::{TCPSender, TCPSenderTrait},
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

#[derive(Debug, PartialEq)]
pub enum TCPSenderStateSummary {
    Error,
    Closed,
    SynSent,
    SynAcked,
    FinSent,
    FinAcked,
}

impl fmt::Display for TCPSenderStateSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TCPSenderStateSummary::Error => write!(f, "error (connection was reset)"),
            TCPSenderStateSummary::Closed => write!(f, "waiting for stream to begin (no SYN sent)"),
            TCPSenderStateSummary::SynSent => write!(f, "stream started but nothing acknowledged"),
            TCPSenderStateSummary::SynAcked => write!(f, "stream ongoing"),
            TCPSenderStateSummary::FinSent => {
                write!(f, "stream finished (FIN sent) but not fully acknowledged")
            }
            TCPSenderStateSummary::FinAcked => write!(f, "stream finished and fully acknowledged"),
        }
    }
}

pub struct TCPState {}

impl TCPState {
    pub fn state_summary_receiver(receiver: &TCPReceiver) -> TCPReceiverStateSummary {
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

    pub fn state_summary_sender(sender: &TCPSender) -> TCPSenderStateSummary {
        if sender.stream_in().borrow().error() {
            return TCPSenderStateSummary::Error;
        } else if sender.next_seqno_absolute() == 0 {
            return TCPSenderStateSummary::Closed;
        } else if sender.next_seqno_absolute() == sender.bytes_in_flight().try_into().unwrap() {
            return TCPSenderStateSummary::SynSent;
        } else if !sender.stream_in().borrow().eof() {
            return TCPSenderStateSummary::SynAcked;
        } else if sender.next_seqno_absolute() < (sender.stream_in().borrow().bytes_written() + 2) {
            return TCPSenderStateSummary::SynAcked;
        } else if sender.bytes_in_flight() != 0 {
            return TCPSenderStateSummary::FinSent;
        } else {
            return TCPSenderStateSummary::FinAcked;
        }
    }
}
