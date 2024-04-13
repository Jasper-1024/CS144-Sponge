mod sender_harness;
use cs144_rust::{
    tcp_helpers::{tcp_config::{TCPConfig, TIMEOUT_DFLT}, tcp_state::TCPSenderStateSummary},
    wrapping_integers::WrappingInt32,
};
use rand::Rng;
use sender_harness::*;

#[test]
fn test_send_close() {
    let mut rng = rand::thread_rng();
    {
        let isn: u32 = rng.gen_range(0..=std::u32::MAX);
        let mut cfg = TCPConfig::new();
        cfg.fixed_isn = Some(WrappingInt32::new(isn));

        let mut test = TCPSenderTestHarness::new("FIN sent test", cfg);
        test.execute(
            &ExpectSegment::new()
                .no_flags()
                .syn(true)
                .payload_size(0)
                .seqno(isn),
        );
        execute_test!(test, AckReceived, WrappingInt32::new(isn + 1));
        execute_test!(test, ExpectState, TCPSenderStateSummary::SynAcked);
        execute_test!(test, Close, true);
        execute_test!(test, ExpectState, TCPSenderStateSummary::FinSent);
        execute_test!(test, ExpectBytesInFlight, 1);
        test.execute(&ExpectSegment::new().fin(true).seqno(isn + 1));
        execute_test!(test, ExpectNoSegment, true);
    }

    {
        let isn: u32 = rng.gen_range(0..=std::u32::MAX);
        let mut cfg = TCPConfig::new();
        cfg.fixed_isn = Some(WrappingInt32::new(isn));

        let mut test = TCPSenderTestHarness::new("FIN acked test", cfg);
        test.execute(
            &ExpectSegment::new()
                .no_flags()
                .syn(true)
                .payload_size(0)
                .seqno(isn),
        );
        execute_test!(test, AckReceived, WrappingInt32::new(isn + 1));
        execute_test!(test, ExpectState, TCPSenderStateSummary::SynAcked);
        execute_test!(test, Close, true);
        execute_test!(test, ExpectState, TCPSenderStateSummary::FinSent);
        test.execute(&ExpectSegment::new().fin(true).seqno(isn + 1));
        execute_test!(test, AckReceived, WrappingInt32::new(isn + 2));
        execute_test!(test, ExpectState, TCPSenderStateSummary::FinAcked);
        execute_test!(test, ExpectBytesInFlight, 0);
        execute_test!(test, ExpectNoSegment, true);
    }

    {
        let isn: u32 = rng.gen_range(0..=std::u32::MAX);
        let mut cfg = TCPConfig::new();
        cfg.fixed_isn = Some(WrappingInt32::new(isn));

        let mut test = TCPSenderTestHarness::new("FIN not acked test", cfg);
        test.execute(
            &ExpectSegment::new()
                .no_flags()
                .syn(true)
                .payload_size(0)
                .seqno(isn),
        );
        execute_test!(test, AckReceived, WrappingInt32::new(isn + 1));
        execute_test!(test, ExpectState, TCPSenderStateSummary::SynAcked);
        execute_test!(test, Close, true);
        execute_test!(test, ExpectState, TCPSenderStateSummary::FinSent);
        test.execute(&ExpectSegment::new().fin(true).seqno(isn + 1));
        execute_test!(test, AckReceived, WrappingInt32::new(isn + 1));
        execute_test!(test, ExpectState, TCPSenderStateSummary::FinSent);
        execute_test!(test, ExpectBytesInFlight, 1);
        execute_test!(test, ExpectNoSegment, true);
    }

    {
        let isn: u32 = rng.gen_range(0..=std::u32::MAX);
        let mut cfg = TCPConfig::new();
        cfg.fixed_isn = Some(WrappingInt32::new(isn));

        let mut test = TCPSenderTestHarness::new("FIN retx test", cfg);
        test.execute(
            &ExpectSegment::new()
                .no_flags()
                .syn(true)
                .payload_size(0)
                .seqno(isn),
        );
        execute_test!(test, AckReceived, WrappingInt32::new(isn + 1));
        execute_test!(test, ExpectState, TCPSenderStateSummary::SynAcked);
        execute_test!(test, Close, true);
        execute_test!(test, ExpectState, TCPSenderStateSummary::FinSent);
        test.execute(&ExpectSegment::new().fin(true).seqno(isn + 1));
        execute_test!(test, AckReceived, WrappingInt32::new(isn + 1));
        execute_test!(test, ExpectState, TCPSenderStateSummary::FinSent);
        execute_test!(test, ExpectBytesInFlight, 1);
        execute_test!(test, ExpectNoSegment, true);
        test.execute(&Tick::new(TIMEOUT_DFLT - 1));
        execute_test!(test, ExpectState, TCPSenderStateSummary::FinSent);
        execute_test!(test, ExpectBytesInFlight, 1);
        execute_test!(test, ExpectNoSegment, true);
        test.execute(&Tick::new(1));
        execute_test!(test, ExpectState, TCPSenderStateSummary::FinSent);
        execute_test!(test, ExpectBytesInFlight, 1);
        test.execute(&ExpectSegment::new().fin(true).seqno(isn + 1));
        execute_test!(test, ExpectNoSegment, true);
        test.execute(&Tick::new(1));
        execute_test!(test, ExpectState, TCPSenderStateSummary::FinSent);
        execute_test!(test, ExpectBytesInFlight, 1);
        execute_test!(test, ExpectNoSegment, true);
        execute_test!(test, AckReceived, WrappingInt32::new(isn + 2));
        execute_test!(test, ExpectState, TCPSenderStateSummary::FinAcked);
        execute_test!(test, ExpectBytesInFlight, 0);
        execute_test!(test, ExpectNoSegment, true);
    }
}
