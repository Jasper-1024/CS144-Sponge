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

/// \brief Summary of a TCPConnection's internal state
///
/// Most TCP implementations have a global per-connection state
/// machine, as described in the [TCP](\ref rfc::rfc793)
/// specification. Sponge is a bit different: we have factored the
/// connection into two independent parts (the sender and the
/// receiver). The TCPSender and TCPReceiver maintain their interval
/// state variables independently (e.g. next_seqno, number of bytes in
/// flight, or whether each stream has ended). There is no notion of a
/// discrete state machine or much overarching state outside the
/// sender and receiver. To test that Sponge follows the TCP spec, we
/// use this class to compare the "official" states with Sponge's
/// sender/receiver states and two variables that belong to the
/// overarching TCPConnection object.
pub struct TCPState {
    sender: String,
    receiver: String,
    active: bool,                      // default is true
    linger_after_streams_finish: bool, // default is false
}

// TCPState 的 内部枚举
pub enum State {
    Listen,      // Listening for a peer to connect = 0
    SynRcvd,     // Got the peer's SYN
    SynSent,     // Sent a SYN to initiate a connection
    Established, // Three-way handshake complete
    CloseWait,   // Remote side has sent a FIN, connection is half-open
    LastAck,     // Local side sent a FIN from CLOSE_WAIT, waiting for ACK
    FinWait1,    // Sent a FIN to the remote side, not yet ACK'd
    FinWait2,    // Received an ACK for previously-sent FIN
    Closing,     // Received a FIN just after we sent one
    TimeWait,    // Both sides have sent FIN and ACK'd, waiting for 2 MSL
    Closed,      // A connection that has terminated normally
    Reset,       // A connection that terminated abnormally
}

impl PartialEq for TCPState {
    fn eq(&self, other: &Self) -> bool {
        self.sender == other.sender
            && self.receiver == other.receiver
            && self.active == other.active
            && self.linger_after_streams_finish == other.linger_after_streams_finish
    }
}

// lab4 add
impl TCPState {
    /// brief Summarize the TCPState in a string
    pub fn name(&self) -> String {
        format!(
            "sender=`{}`, receiver=`{}`, active={}, linger_after_streams_finish={}",
            self.sender, self.receiver, self.active, self.linger_after_streams_finish
        )
    }
    /// Construct a TCPState given a sender, a receiver, and the TCPConnection's active and linger bits
    pub fn new(sender: &TCPSender, receiver: &TCPReceiver, active: bool, linger: bool) -> Self {
        TCPState {
            sender: TCPState::state_summary_sender(sender).to_string(),
            receiver: TCPState::state_summary_receiver(receiver).to_string(),
            active: active,
            linger_after_streams_finish: linger,
        }
    }
    /// Construct a TCPState that corresponds to one of the "official" TCP state names
    pub fn new_from(state: State) -> Self {
        let (receiver, sender, linger_after_streams_finish, active) = match state {
            State::Listen => (
                TCPReceiverStateSummary::Listen,
                TCPSenderStateSummary::Closed,
                true,
                true,
            ),
            State::SynRcvd => (
                TCPReceiverStateSummary::SynReceived,
                TCPSenderStateSummary::SynSent,
                true,
                true,
            ),
            State::SynSent => (
                TCPReceiverStateSummary::Listen,
                TCPSenderStateSummary::SynSent,
                true,
                true,
            ),
            State::Established => (
                TCPReceiverStateSummary::SynReceived,
                TCPSenderStateSummary::SynAcked,
                true,
                true,
            ),
            State::CloseWait => (
                TCPReceiverStateSummary::FinReceived,
                TCPSenderStateSummary::SynAcked,
                false,
                true,
            ),
            State::LastAck => (
                TCPReceiverStateSummary::FinReceived,
                TCPSenderStateSummary::FinSent,
                false,
                true,
            ),
            State::Closing => (
                TCPReceiverStateSummary::FinReceived,
                TCPSenderStateSummary::FinSent,
                true,
                true,
            ),
            State::FinWait1 => (
                TCPReceiverStateSummary::SynReceived,
                TCPSenderStateSummary::FinSent,
                true,
                true,
            ),
            State::FinWait2 => (
                TCPReceiverStateSummary::SynReceived,
                TCPSenderStateSummary::FinAcked,
                true,
                true,
            ),
            State::TimeWait => (
                TCPReceiverStateSummary::FinReceived,
                TCPSenderStateSummary::FinAcked,
                true,
                true,
            ),
            State::Reset => (
                TCPReceiverStateSummary::Error,
                TCPSenderStateSummary::Error,
                false,
                false,
            ),
            State::Closed => (
                TCPReceiverStateSummary::FinReceived,
                TCPSenderStateSummary::FinAcked,
                false,
                false,
            ),
        };

        TCPState {
            sender: sender.to_string(),
            receiver: receiver.to_string(),
            linger_after_streams_finish,
            active,
        }
    }
}

impl TCPState {
    /// \brief Summarize the state of a TCPReceiver in a string
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

    /// \brief Summarize the state of a TCPSender in a string
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
