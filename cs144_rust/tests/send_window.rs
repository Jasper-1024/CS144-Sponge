mod sender_harness;
use cs144_rust::{
    tcp_helpers::{
        tcp_config::{TCPConfig, MAX_RETX_ATTEMPTS},
        tcp_state::TCPSenderStateSummary,
    },
    wrapping_integers::WrappingInt32,
};
use rand::Rng;
use sender_harness::*;

#[test]
fn test_send_window() {
    let mut rng = rand::thread_rng();
    {
        let isn: u32 = rng.gen_range(0..=std::u32::MAX);
        let mut cfg = TCPConfig::new();
        cfg.fixed_isn = Some(WrappingInt32::new(isn));

        let mut test =
            TCPSenderTestHarness::new("Initial receiver advertised window is respected", cfg);
        test.execute(
            &ExpectSegment::new()
                .no_flags()
                .syn(true)
                .payload_size(0)
                .seqno(isn),
        );
        test.execute(&AckReceived::new(WrappingInt32::new(isn + 1)).with_win(4));
        execute_test!(test, ExpectNoSegment, true);
        test.execute(&WriteBytes::new("abcdefg".into(), false));
        test.execute(
            &ExpectSegment::new()
                .no_flags()
                .data("abcd".into())
                .seqno(isn + 1),
        );
        execute_test!(test, ExpectNoSegment, true);
    }
    {
        let isn: u32 = rng.gen_range(0..=std::u32::MAX);
        let mut cfg = TCPConfig::new();
        cfg.fixed_isn = Some(WrappingInt32::new(isn));

        let mut test = TCPSenderTestHarness::new("Immediate window is respected", cfg);
        test.execute(
            &ExpectSegment::new()
                .no_flags()
                .syn(true)
                .payload_size(0)
                .seqno(isn),
        );
        test.execute(&AckReceived::new(WrappingInt32::new(isn + 1)).with_win(6));
        execute_test!(test, ExpectNoSegment, true);
        test.execute(&WriteBytes::new("abcdefg".into(), false));
        test.execute(
            &ExpectSegment::new()
                .no_flags()
                .data("abcdef".into())
                .seqno(isn + 1),
        );
        execute_test!(test, ExpectNoSegment, true);
    }
    {
        let isn: u32 = rng.gen_range(0..=std::u32::MAX);
        let mut cfg = TCPConfig::new();
        cfg.fixed_isn = Some(WrappingInt32::new(isn));

        let mut test = TCPSenderTestHarness::new("Window filling", cfg);
        test.execute(
            &ExpectSegment::new()
                .no_flags()
                .syn(true)
                .payload_size(0)
                .seqno(isn),
        );
        test.execute(&AckReceived::new(WrappingInt32::new(isn + 1)).with_win(3));
        execute_test!(test, ExpectState, TCPSenderStateSummary::SynAcked);
        test.execute(&WriteBytes::new("01234567".into(), false));
        execute_test!(test, ExpectBytesInFlight, 3);
        test.execute(&ExpectSegment::new().data("012".into()).seqno(isn + 1));
        execute_test!(test, ExpectNoSegment, true);
        execute_test!(test, ExpectSeqno, WrappingInt32::new(isn + 1 + 3));
        test.execute(&AckReceived::new(WrappingInt32::new(isn + 1 + 3)).with_win(3));
        execute_test!(test, ExpectBytesInFlight, 3);
        test.execute(&ExpectSegment::new().data("345".into()).seqno(isn + 1 + 3));
        execute_test!(test, ExpectNoSegment, true);
        execute_test!(test, ExpectSeqno, WrappingInt32::new(isn + 1 + 6));
        test.execute(&AckReceived::new(WrappingInt32::new(isn + 1 + 6)).with_win(3));
        execute_test!(test, ExpectBytesInFlight, 2);
        test.execute(&ExpectSegment::new().data("67".into()).seqno(isn + 1 + 6));
        execute_test!(test, ExpectNoSegment, true);
        execute_test!(test, ExpectSeqno, WrappingInt32::new(isn + 1 + 8));
        test.execute(&AckReceived::new(WrappingInt32::new(isn + 1 + 8)).with_win(3));
        execute_test!(test, ExpectBytesInFlight, 0);
        execute_test!(test, ExpectNoSegment, true);
    }
    {
        let isn: u32 = rng.gen_range(0..=std::u32::MAX);
        let mut cfg = TCPConfig::new();
        cfg.fixed_isn = Some(WrappingInt32::new(isn));

        let mut test = TCPSenderTestHarness::new("Immediate writes respect the window", cfg);
        test.execute(
            &ExpectSegment::new()
                .no_flags()
                .syn(true)
                .payload_size(0)
                .seqno(isn),
        );
        test.execute(&AckReceived::new(WrappingInt32::new(isn + 1)).with_win(3));
        execute_test!(test, ExpectState, TCPSenderStateSummary::SynAcked);
        test.execute(&WriteBytes::new("01".into(), false));
        execute_test!(test, ExpectBytesInFlight, 2);
        test.execute(&ExpectSegment::new().data("01".into()).seqno(isn + 1));
        execute_test!(test, ExpectNoSegment, true);
        execute_test!(test, ExpectSeqno, WrappingInt32::new(isn + 1 + 2));
        test.execute(&WriteBytes::new("23".into(), false));
        execute_test!(test, ExpectBytesInFlight, 3);
        test.execute(&ExpectSegment::new().data("2".into()).seqno(isn + 1 + 2));
        execute_test!(test, ExpectNoSegment, true);
        execute_test!(test, ExpectSeqno, WrappingInt32::new(isn + 1 + 3));
    }
    {
        let mut rng = rand::thread_rng();
        let isn: u32 = rng.gen_range(0..=std::u32::MAX);
        let mut cfg = TCPConfig::new();
        cfg.fixed_isn = Some(WrappingInt32::new(isn));

        let mut test = TCPSenderTestHarness::new("FIN flag occupies space in window", cfg);
        test.execute(
            &ExpectSegment::new()
                .no_flags()
                .syn(true)
                .payload_size(0)
                .seqno(isn),
        );
        test.execute(&AckReceived::new(WrappingInt32::new(isn + 1)).with_win(7));
        execute_test!(test, ExpectNoSegment, true);
        test.execute(&WriteBytes::new("1234567".into(), false));
        test.execute(
            &ExpectSegment::new()
                .no_flags()
                .data("1234567".into())
                .seqno(isn + 1),
        );
        execute_test!(test, ExpectNoSegment, true);
        test.execute(&Close {});
        test.execute(&AckReceived::new(WrappingInt32::new(isn + 8)).with_win(1));
        test.execute(
            &ExpectSegment::new()
                .fin(true)
                .data("".into())
                .seqno(isn + 8),
        );
        execute_test!(test, ExpectNoSegment, true);
    }
    {
        let mut rng = rand::thread_rng();
        let isn: u32 = rng.gen_range(0..=std::u32::MAX);
        let mut cfg = TCPConfig::new();
        cfg.fixed_isn = Some(WrappingInt32::new(isn));

        let mut test =
            TCPSenderTestHarness::new("Piggyback FIN in segment when space is available", cfg);
        test.execute(
            &ExpectSegment::new()
                .no_flags()
                .syn(true)
                .payload_size(0)
                .seqno(isn),
        );
        test.execute(&AckReceived::new(WrappingInt32::new(isn + 1)).with_win(3));
        execute_test!(test, ExpectState, TCPSenderStateSummary::SynAcked);
        test.execute(&WriteBytes::new("1234567".into(), false));
        test.execute(
            &ExpectSegment::new()
                .no_flags()
                .data("123".into())
                .seqno(isn + 1),
        );
        execute_test!(test, ExpectNoSegment, true);
        test.execute(&Close{});
        test.execute(&AckReceived::new(WrappingInt32::new(isn + 1)).with_win(8));
        test.execute(
            &ExpectSegment::new()
                .fin(true)
                .data("4567".into())
                .seqno(isn + 1 + 3), // Adding 3 because "123" was previously sent.
        );
        execute_test!(test, ExpectNoSegment, true);
    }
}
