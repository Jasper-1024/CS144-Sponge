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

        execute_test!(test, ExpectState, TCPSenderStateSummary::SynSent);
        test.execute(
            &ExpectSegment::new()
                .no_flags()
                .syn(true)
                .payload_size(0)
                .seqno(isn),
        );
        execute_test!(test, ExpectBytesInFlight, 1);
    }
    {
        let isn: u32 = rng.gen_range(0..=std::u32::MAX);
        let mut cfg = TCPConfig::new();
        cfg.fixed_isn = Some(WrappingInt32::new(isn));

        let mut test = TCPSenderTestHarness::new("SYN sent test", cfg);
        execute_test!(test, ExpectState, TCPSenderStateSummary::SynSent);
        test.execute(
            &ExpectSegment::new()
                .no_flags()
                .syn(true)
                .payload_size(0)
                .seqno(isn),
        );
        execute_test!(test, ExpectBytesInFlight, 1);
        execute_test!(test, AckReceived, WrappingInt32::new(isn + 1));
        execute_test!(test, ExpectState, TCPSenderStateSummary::SynAcked);
        execute_test!(test, ExpectNoSegment, true);
        execute_test!(test, ExpectBytesInFlight, 0);
    }
    {
        let isn: u32 = rng.gen_range(0..=std::u32::MAX);
        let mut cfg = TCPConfig::new();
        cfg.fixed_isn = Some(WrappingInt32::new(isn));

        let mut test = TCPSenderTestHarness::new("SYN -> wrong ack test", cfg);
        execute_test!(test, ExpectState, TCPSenderStateSummary::SynSent);
        test.execute(
            &ExpectSegment::new()
                .no_flags()
                .syn(true)
                .payload_size(0)
                .seqno(isn),
        );
        execute_test!(test, ExpectBytesInFlight, 1);
        execute_test!(test, AckReceived, WrappingInt32::new(isn));
        execute_test!(test, ExpectState, TCPSenderStateSummary::SynSent);
        execute_test!(test, ExpectNoSegment, true);
        execute_test!(test, ExpectBytesInFlight, 1);
    }

    {
        let isn: u32 = rng.gen_range(0..=std::u32::MAX);
        let mut cfg = TCPConfig::new();
        cfg.fixed_isn = Some(WrappingInt32::new(isn));

        let mut test = TCPSenderTestHarness::new("SYN acked, data", cfg);
        execute_test!(test, ExpectState, TCPSenderStateSummary::SynSent);
        test.execute(
            &ExpectSegment::new()
                .no_flags()
                .syn(true)
                .payload_size(0)
                .seqno(isn),
        );
        execute_test!(test, ExpectBytesInFlight, 1);
        execute_test!(test, AckReceived, WrappingInt32::new(isn + 1));
        execute_test!(test, ExpectState, TCPSenderStateSummary::SynAcked);
        execute_test!(test, ExpectNoSegment, true);
        execute_test!(test, ExpectBytesInFlight, 0);
        test.execute(&WriteBytes::new("abcdefgh".into(), false));
        test.execute(&Tick::new(1));
        execute_test!(test, ExpectState, TCPSenderStateSummary::SynAcked);
        test.execute(&ExpectSegment::new().seqno(isn + 1).data("abcdefgh".into()));
        execute_test!(test, ExpectBytesInFlight, 8);
        execute_test!(test, AckReceived, WrappingInt32::new(isn + 9));
        execute_test!(test, ExpectState, TCPSenderStateSummary::SynAcked);
        execute_test!(test, ExpectNoSegment, true);
        execute_test!(test, ExpectBytesInFlight, 0);
        execute_test!(test, ExpectSeqno, WrappingInt32::new(isn + 9));
    }
}
