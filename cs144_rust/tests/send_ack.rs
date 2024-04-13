mod sender_harness;
use cs144_rust::{
    tcp_helpers::{tcp_config::TCPConfig, tcp_state::TCPSenderStateSummary},
    wrapping_integers::WrappingInt32,
};
use rand::Rng;
use sender_harness::*;

#[test]
fn test_ack() {
    {
        let mut rng = rand::thread_rng();
        {
            let isn: u32 = rng.gen_range(0..=std::u32::MAX);
            let mut cfg = TCPConfig::new();
            cfg.fixed_isn = Some(WrappingInt32::new(isn));

            let mut test = TCPSenderTestHarness::new("Repeat ACK is ignored", cfg);
            test.execute(
                &ExpectSegment::new()
                    .no_flags()
                    .syn(true)
                    .payload_size(0)
                    .seqno(isn),
            );
            execute_test!(test, ExpectNoSegment, true);
            execute_test!(test, AckReceived, WrappingInt32::new(isn + 1));
            test.execute(&WriteBytes::new("a".into(), false));
            test.execute(&ExpectSegment::new().no_flags().data("a".into()));
            execute_test!(test, ExpectNoSegment, true);
            execute_test!(test, AckReceived, WrappingInt32::new(isn + 1));
            execute_test!(test, ExpectNoSegment, true);
        }

        {
            let isn: u32 = rng.gen_range(0..=std::u32::MAX);
            let mut cfg = TCPConfig::new();
            cfg.fixed_isn = Some(WrappingInt32::new(isn));

            let mut test = TCPSenderTestHarness::new("Old ACK is ignored", cfg);
            test.execute(
                &ExpectSegment::new()
                    .no_flags()
                    .syn(true)
                    .payload_size(0)
                    .seqno(isn),
            );
            execute_test!(test, ExpectNoSegment, true);
            execute_test!(test, AckReceived, WrappingInt32::new(isn + 1));
            test.execute(&WriteBytes::new("a".into(), false));
            test.execute(&ExpectSegment::new().no_flags().data("a".into()));
            execute_test!(test, ExpectNoSegment, true);
            execute_test!(test, AckReceived, WrappingInt32::new(isn + 2));
            execute_test!(test, ExpectNoSegment, true);
            test.execute(&WriteBytes::new("b".into(), false));
            test.execute(&ExpectSegment::new().no_flags().data("b".into()));
            execute_test!(test, ExpectNoSegment, true);
            execute_test!(test, AckReceived, WrappingInt32::new(isn + 1));
            execute_test!(test, ExpectNoSegment, true);
        }

        // credit for test: Jared Wasserman (2020)
        {
            let isn: u32 = rng.gen_range(0..=std::u32::MAX);
            let mut cfg = TCPConfig::new();
            cfg.fixed_isn = Some(WrappingInt32::new(isn));

            let mut test =
                TCPSenderTestHarness::new("Impossible ackno (beyond next seqno) is ignored", cfg);
            execute_test!(test, ExpectState, TCPSenderStateSummary::SynSent);
            test.execute(
                &ExpectSegment::new()
                    .no_flags()
                    .syn(true)
                    .payload_size(0)
                    .seqno(isn),
            );
            test.execute(&AckReceived::new(WrappingInt32::new(isn + 2)).with_win(1000));
            execute_test!(test, ExpectState, TCPSenderStateSummary::SynSent);
        }
    }
}
