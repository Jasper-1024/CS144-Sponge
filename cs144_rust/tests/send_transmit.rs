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
fn test_send_transmit() {
    let mut rng = rand::thread_rng();
    {
        let isn: u32 = rng.gen_range(0..=std::u32::MAX);
        let mut cfg = TCPConfig::new();
        cfg.fixed_isn = Some(WrappingInt32::new(isn));

        let mut test = TCPSenderTestHarness::new("Three short writes", cfg);
        test.execute(
            &ExpectSegment::new()
                .no_flags()
                .syn(true)
                .payload_size(0)
                .seqno(isn),
        );
        test.execute(&AckReceived::new(WrappingInt32::new(isn + 1)));
        execute_test!(test, ExpectState, TCPSenderStateSummary::SynAcked);
        test.execute(&WriteBytes::new("ab".into(), false));
        test.execute(&ExpectSegment::new().data("ab".into()).seqno(isn + 1));
        test.execute(&WriteBytes::new("cd".into(), false));
        test.execute(&ExpectSegment::new().data("cd".into()).seqno(isn + 3));
        test.execute(&WriteBytes::new("abcd".into(), false));
        test.execute(&ExpectSegment::new().data("abcd".into()).seqno(isn + 5));
        execute_test!(test, ExpectSeqno, WrappingInt32::new(isn + 9));
        execute_test!(test, ExpectBytesInFlight, 8);
        execute_test!(test, ExpectState, TCPSenderStateSummary::SynAcked);
    }
    {
        let isn: u32 = rng.gen_range(0..=std::u32::MAX);
        let mut cfg = TCPConfig::new();
        cfg.fixed_isn = Some(WrappingInt32::new(isn));

        let mut test = TCPSenderTestHarness::new("Many short writes, continuous acks", cfg);
        test.execute(
            &ExpectSegment::new()
                .no_flags()
                .syn(true)
                .payload_size(0)
                .seqno(isn),
        );
        test.execute(&AckReceived::new(WrappingInt32::new(isn + 1)));
        execute_test!(test, ExpectState, TCPSenderStateSummary::SynAcked);

        let max_block_size = 10;
        let n_rounds = 10000;
        let mut bytes_sent = 0;
        for i in 0..n_rounds {
            let block_size = rng.gen_range(1..=max_block_size);
            let data: String = (0..block_size)
                .map(|j| {
                    let c = char::from(b'a' + ((i + j) % 26) as u8);
                    c
                })
                .collect();
            test.execute(&ExpectSeqno::new(WrappingInt32::new(
                isn + bytes_sent as u32 + 1,
            )));
            test.execute(&WriteBytes::new(data.clone().into(), false));
            bytes_sent += block_size;
            execute_test!(test, ExpectBytesInFlight, block_size);
            test.execute(
                &ExpectSegment::new()
                    .data(data.into())
                    .seqno(isn + 1 + bytes_sent as u32 - block_size as u32),
            );
            test.execute(&ExpectNoSegment {});
            test.execute(&AckReceived::new(WrappingInt32::new(
                isn + 1 + bytes_sent as u32,
            )));
        }
    }
    {
        // Setup initial conditions similar to previous tests
        let isn: u32 = rng.gen_range(0..=std::u32::MAX);
        let mut cfg = TCPConfig::new();
        cfg.fixed_isn = Some(WrappingInt32::new(isn));

        let mut test = TCPSenderTestHarness::new("Many short writes, ack at end", cfg);
        test.execute(
            &ExpectSegment::new()
                .no_flags()
                .syn(true)
                .payload_size(0)
                .seqno(isn),
        );
        test.execute(&AckReceived::new(WrappingInt32::new(isn + 1)).with_win(65000));
        execute_test!(test, ExpectState, TCPSenderStateSummary::SynAcked);

        let max_block_size = 10;
        let n_rounds = 1000;
        let mut bytes_sent = 0;
        for i in 0..n_rounds {
            let block_size = rng.gen_range(1..=max_block_size);
            let data: String = (0..block_size)
                .map(|j| {
                    let c = char::from(b'a' + ((i + j) % 26) as u8);
                    c
                })
                .collect();
            test.execute(&ExpectSeqno::new(WrappingInt32::new(
                isn + bytes_sent as u32 + 1,
            )));
            test.execute(&WriteBytes::new(data.clone().into(), false));
            bytes_sent += block_size;
            execute_test!(test, ExpectBytesInFlight, bytes_sent);

            test.execute(
                &ExpectSegment::new()
                    .data(data.into())
                    .seqno(isn + 1 + bytes_sent as u32 - block_size as u32),
            );
            test.execute(&ExpectNoSegment {});
        }
        execute_test!(test, ExpectBytesInFlight, bytes_sent);
        test.execute(&AckReceived::new(WrappingInt32::new(
            isn + 1 + bytes_sent as u32,
        )));
        execute_test!(test, ExpectBytesInFlight, 0);
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
        test.execute(&ExpectNoSegment {});
        execute_test!(test, ExpectSeqno, WrappingInt32::new(isn + 1 + 2));
        test.execute(&WriteBytes::new("23".into(), false));
        execute_test!(test, ExpectBytesInFlight, 3);
        test.execute(&ExpectSegment::new().data("2".into()).seqno(isn + 1 + 2));
        test.execute(&ExpectNoSegment {});
        execute_test!(test, ExpectSeqno, WrappingInt32::new(isn + 1 + 3));
    }
}
