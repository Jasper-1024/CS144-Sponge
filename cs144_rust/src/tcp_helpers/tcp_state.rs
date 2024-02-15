pub enum TCPReceiverStateSummary {
    Error,
    Listen,
    SynReceived,
    FinReceived,
}

impl ToString for TCPReceiverStateSummary {
    fn to_string(&self) -> String {
        match self {
            TCPReceiverStateSummary::Error => "error (connection was reset)".to_string(),
            TCPReceiverStateSummary::Listen => "waiting for SYN: ackno is empty".to_string(),
            TCPReceiverStateSummary::SynReceived => {
                "SYN received (ackno exists), and input to stream hasn't ended".to_string()
            }
            TCPReceiverStateSummary::FinReceived => "input to stream has ended".to_string(),
        }
    }
}
