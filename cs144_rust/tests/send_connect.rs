mod sender_harness;
use cs144_rust::{
    tcp_helpers::{tcp_config::TCPConfig, tcp_state::TCPSenderStateSummary},
    wrapping_integers::WrappingInt32,
};
use rand::Rng;
use sender_harness::*;

#[test]
fn test_send_connect() {
    let mut rng = rand::thread_rng();
    {
        let isn: u32 = rng.gen_range(0..=std::u32::MAX);
        let mut cfg = TCPConfig::new();
        cfg.fixed_isn = Some(WrappingInt32::new(isn));

        let mut test = TCPSenderTestHarness::new("SYN sent test", cfg);
        test.execute(&ExpectState::new(TCPSenderStateSummary::SynSent));
        test.execute(
            &SegmentArrives::new()
                .no_flags()
                .syn(true)
                .payload_size(0)
                .seqno(isn),
        );
        test.execute(&ExpectBytesInFlight::new(1));
    }
}
