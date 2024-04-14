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
fn test_send_retx() {
    let mut rng = rand::thread_rng();
    {
        let isn: u32 = rng.gen_range(0..=std::u32::MAX);
        let retx_timeout: u64 = rng.gen_range(10..=10000);
        let mut cfg = TCPConfig::new();
        cfg.fixed_isn = Some(WrappingInt32::new(isn));
        cfg.rt_timeout = retx_timeout;

        let mut test =
            TCPSenderTestHarness::new("Retx SYN twice at the right times, then ack", cfg);
        test.execute(
            &ExpectSegment::new()
                .no_flags()
                .syn(true)
                .payload_size(0)
                .seqno(isn),
        );
        execute_test!(test, ExpectNoSegment, true);
        execute_test!(test, ExpectState, TCPSenderStateSummary::SynSent);
        test.execute(&Tick::new(retx_timeout - 1));
        execute_test!(test, ExpectNoSegment, true);
        test.execute(&Tick::new(1));
        test.execute(
            &ExpectSegment::new()
                .no_flags()
                .syn(true)
                .payload_size(0)
                .seqno(isn),
        );
        execute_test!(test, ExpectState, TCPSenderStateSummary::SynSent);
        execute_test!(test, ExpectBytesInFlight, 1);
        test.execute(&Tick::new(2 * retx_timeout - 1));
        execute_test!(test, ExpectNoSegment, true);
        test.execute(&Tick::new(1));
        test.execute(
            &ExpectSegment::new()
                .no_flags()
                .syn(true)
                .payload_size(0)
                .seqno(isn),
        );
        execute_test!(test, ExpectState, TCPSenderStateSummary::SynSent);
        execute_test!(test, ExpectBytesInFlight, 1);
        test.execute(&AckReceived::new(WrappingInt32::new(isn + 1)));
        execute_test!(test, ExpectState, TCPSenderStateSummary::SynAcked);
        execute_test!(test, ExpectBytesInFlight, 0);
    }
    {
        let isn: u32 = rng.gen_range(0..=std::u32::MAX);
        let retx_timeout: u64 = rng.gen_range(10..=10000);
        let mut cfg = TCPConfig::new();
        cfg.fixed_isn = Some(WrappingInt32::new(isn));
        cfg.rt_timeout = retx_timeout;

        let mut test = TCPSenderTestHarness::new("Retx SYN until too many retransmissions", cfg);
        test.execute(
            &ExpectSegment::new()
                .no_flags()
                .syn(true)
                .payload_size(0)
                .seqno(isn),
        );
        execute_test!(test, ExpectNoSegment, true);
        execute_test!(test, ExpectState, TCPSenderStateSummary::SynSent);
        for attempt_no in 0..MAX_RETX_ATTEMPTS {
            test.execute(
                &Tick::new((retx_timeout << attempt_no) - 1).with_max_retx_exceeded(false),
            );
            execute_test!(test, ExpectNoSegment, true);
            test.execute(&Tick::new(1).with_max_retx_exceeded(false));
            test.execute(
                &ExpectSegment::new()
                    .no_flags()
                    .syn(true)
                    .payload_size(0)
                    .seqno(isn),
            );
            execute_test!(test, ExpectState, TCPSenderStateSummary::SynSent);
            execute_test!(test, ExpectBytesInFlight, 1);
        }
        test.execute(
            &Tick::new((retx_timeout << MAX_RETX_ATTEMPTS) - 1).with_max_retx_exceeded(false),
        );
        test.execute(&Tick::new(1).with_max_retx_exceeded(true));
    }

    {
        let isn: u32 = rng.gen_range(0..=std::u32::MAX);
        let retx_timeout: u64 = rng.gen_range(10..=10000);
        let mut cfg = TCPConfig::new();
        cfg.fixed_isn = Some(WrappingInt32::new(isn));
        cfg.rt_timeout = retx_timeout;

        let mut test = TCPSenderTestHarness::new(
            "Send some data, the retx and succeed, then retx till limit",
            cfg,
        );
        test.execute(
            &ExpectSegment::new()
                .no_flags()
                .syn(true)
                .payload_size(0)
                .seqno(isn),
        );
        execute_test!(test, ExpectNoSegment, true);
        test.execute(&AckReceived::new(WrappingInt32::new(isn + 1)));
        test.execute(&WriteBytes::new("abcd".into(), false));
        test.execute(
            &ExpectSegment::new()
                .payload_size(4)
                .data("abcd".into())
                .seqno(isn + 1),
        );
        execute_test!(test, ExpectNoSegment, true);
        test.execute(&AckReceived::new(WrappingInt32::new(isn + 5)));
        execute_test!(test, ExpectBytesInFlight, 0);

        test.execute(&WriteBytes::new("efgh".into(), false));
        test.execute(
            &ExpectSegment::new()
                .payload_size(4)
                .data("efgh".into())
                .seqno(isn + 5),
        );
        execute_test!(test, ExpectNoSegment, true);
        test.execute(&Tick::new(retx_timeout).with_max_retx_exceeded(false));
        test.execute(
            &ExpectSegment::new()
                .payload_size(4)
                .data("efgh".into())
                .seqno(isn + 5),
        );
        execute_test!(test, ExpectNoSegment, true);
        test.execute(&AckReceived::new(WrappingInt32::new(isn + 9)));
        execute_test!(test, ExpectBytesInFlight, 0);

        test.execute(&WriteBytes::new("ijkl".into(), false));
        test.execute(
            &ExpectSegment::new()
                .payload_size(4)
                .data("ijkl".into())
                .seqno(isn + 9),
        );
        for attempt_no in 0..MAX_RETX_ATTEMPTS {
            test.execute(
                &Tick::new((retx_timeout << attempt_no) - 1).with_max_retx_exceeded(false),
            );
            execute_test!(test, ExpectNoSegment, true);
            test.execute(&Tick::new(1).with_max_retx_exceeded(false));
            test.execute(
                &ExpectSegment::new()
                    .payload_size(4)
                    .data("ijkl".into())
                    .seqno(isn + 9),
            );
            execute_test!(test, ExpectState, TCPSenderStateSummary::SynAcked);
            execute_test!(test, ExpectBytesInFlight, 4);
        }
        test.execute(
            &Tick::new((retx_timeout << MAX_RETX_ATTEMPTS) - 1).with_max_retx_exceeded(false),
        );
        test.execute(&Tick::new(1).with_max_retx_exceeded(true));
    }
}
