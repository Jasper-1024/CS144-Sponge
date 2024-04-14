mod sender_harness;
use cs144_rust::{
    tcp_helpers::{
        tcp_config::{TCPConfig, DEFAULT_CAPACITY, MAX_PAYLOAD_SIZE, TIMEOUT_DFLT},
        tcp_state::TCPSenderStateSummary,
    },
    wrapping_integers::WrappingInt32,
};
use rand::Rng;
use sender_harness::*;

#[test]
fn test_send_extra() {
    let mut rng = rand::thread_rng();
    {
        let isn: u32 = rng.gen_range(0..=std::u32::MAX);
        let rto: u64 = rng.gen_range(30..=10000);
        let mut cfg = TCPConfig::new();
        cfg.fixed_isn = Some(WrappingInt32::new(isn));
        cfg.rt_timeout = rto;

        let mut test = TCPSenderTestHarness::new(
            "If already running, timer stays running when new segment sent",
            cfg,
        );
        test.execute(
            &ExpectSegment::new()
                .no_flags()
                .syn(true)
                .payload_size(0)
                .seqno(isn),
        );
        test.execute(&AckReceived::new(WrappingInt32::new(isn + 1)).with_win(1000));
        execute_test!(test, ExpectState, TCPSenderStateSummary::SynAcked);
        test.execute(&WriteBytes::new("abc".into(), false));
        test.execute(
            &ExpectSegment::new()
                .payload_size(3)
                .data("abc".into())
                .seqno(isn + 1),
        );
        test.execute(&Tick::new(rto - 5));
        execute_test!(test, ExpectNoSegment, true);
        test.execute(&WriteBytes::new("def".into(), false));
        test.execute(&ExpectSegment::new().payload_size(3).data("def".into()));
        test.execute(&Tick::new(6));
        test.execute(
            &ExpectSegment::new()
                .payload_size(3)
                .data("abc".into())
                .seqno(isn + 1),
        );
        execute_test!(test, ExpectNoSegment, true);
    }
    {
        let isn: u32 = rng.gen_range(0..=std::u32::MAX);
        let rto: u64 = rng.gen_range(30..=10000);
        let mut cfg = TCPConfig::new();
        cfg.fixed_isn = Some(WrappingInt32::new(isn));
        cfg.rt_timeout = rto;

        let mut test = TCPSenderTestHarness::new(
            "Retransmission still happens when expiration time not hit exactly",
            cfg,
        );
        test.execute(
            &ExpectSegment::new()
                .no_flags()
                .syn(true)
                .payload_size(0)
                .seqno(isn),
        );
        test.execute(&AckReceived::new(WrappingInt32::new(isn + 1)).with_win(1000));
        execute_test!(test, ExpectState, TCPSenderStateSummary::SynAcked);
        test.execute(&WriteBytes::new("abc".into(), false));
        test.execute(
            &ExpectSegment::new()
                .payload_size(3)
                .data("abc".into())
                .seqno(isn + 1),
        );
        test.execute(&Tick::new(rto - 5));
        execute_test!(test, ExpectNoSegment, true);
        test.execute(&WriteBytes::new("def".into(), false));
        test.execute(&ExpectSegment::new().payload_size(3).data("def".into()));
        test.execute(&Tick::new(200));
        test.execute(
            &ExpectSegment::new()
                .payload_size(3)
                .data("abc".into())
                .seqno(isn + 1),
        );
        execute_test!(test, ExpectNoSegment, true);
    }
    {
        let isn: u32 = rng.gen_range(0..=std::u32::MAX);
        let rto: u64 = rng.gen_range(30..=10000);
        let mut cfg = TCPConfig::new();
        cfg.fixed_isn = Some(WrappingInt32::new(isn));
        cfg.rt_timeout = rto;

        let mut test = TCPSenderTestHarness::new("Timer restarts on ACK of new data", cfg);
        test.execute(
            &ExpectSegment::new()
                .no_flags()
                .syn(true)
                .payload_size(0)
                .seqno(isn),
        );
        test.execute(&AckReceived::new(WrappingInt32::new(isn + 1)).with_win(1000));
        execute_test!(test, ExpectState, TCPSenderStateSummary::SynAcked);
        test.execute(&WriteBytes::new("abc".into(), false));
        test.execute(
            &ExpectSegment::new()
                .payload_size(3)
                .data("abc".into())
                .seqno(isn + 1),
        );
        test.execute(&Tick::new(rto - 5));
        test.execute(&WriteBytes::new("def".into(), false));
        test.execute(
            &ExpectSegment::new()
                .payload_size(3)
                .data("def".into())
                .seqno(isn + 4),
        );
        test.execute(&AckReceived::new(WrappingInt32::new(isn + 4)).with_win(1000));
        test.execute(&Tick::new(rto - 1));
        execute_test!(test, ExpectNoSegment, true);
        test.execute(&Tick::new(2));
        test.execute(
            &ExpectSegment::new()
                .payload_size(3)
                .data("def".into())
                .seqno(isn + 4),
        );
    }
    {
        let isn: u32 = rng.gen_range(0..=std::u32::MAX);
        let rto: u64 = rng.gen_range(30..=10000);
        let mut cfg = TCPConfig::new();
        cfg.fixed_isn = Some(WrappingInt32::new(isn));
        cfg.rt_timeout = rto;

        let mut test =
            TCPSenderTestHarness::new("Timer doesn't restart without ACK of new data", cfg);
        test.execute(
            &ExpectSegment::new()
                .no_flags()
                .syn(true)
                .payload_size(0)
                .seqno(isn),
        );
        test.execute(&AckReceived::new(WrappingInt32::new(isn + 1)).with_win(1000));
        execute_test!(test, ExpectState, TCPSenderStateSummary::SynAcked);
        test.execute(&WriteBytes::new("abc".into(), false));
        test.execute(
            &ExpectSegment::new()
                .payload_size(3)
                .data("abc".into())
                .seqno(isn + 1),
        );
        test.execute(&Tick::new(rto - 5));
        test.execute(&WriteBytes::new("def".into(), false));
        test.execute(
            &ExpectSegment::new()
                .payload_size(3)
                .data("def".into())
                .seqno(isn + 4),
        );
        test.execute(&AckReceived::new(WrappingInt32::new(isn + 1)).with_win(1000));
        test.execute(&Tick::new(6));
        test.execute(
            &ExpectSegment::new()
                .payload_size(3)
                .data("abc".into())
                .seqno(isn + 1),
        );
        execute_test!(test, ExpectNoSegment, true);
        test.execute(&Tick::new(rto * 2 - 5));
        execute_test!(test, ExpectNoSegment, true);
        test.execute(&Tick::new(8));
        test.execute(
            &ExpectSegment::new()
                .payload_size(3)
                .data("abc".into())
                .seqno(isn + 1),
        );
        execute_test!(test, ExpectNoSegment, true);
    }
    {
        let isn: u32 = rng.gen_range(0..=std::u32::MAX);
        let rto: u64 = rng.gen_range(30..=10000);
        let mut cfg = TCPConfig::new();
        cfg.fixed_isn = Some(WrappingInt32::new(isn));
        cfg.rt_timeout = rto;

        let mut test = TCPSenderTestHarness::new("RTO resets on ACK of new data", cfg);
        test.execute(
            &ExpectSegment::new()
                .no_flags()
                .syn(true)
                .payload_size(0)
                .seqno(isn),
        );
        test.execute(&AckReceived::new(WrappingInt32::new(isn + 1)).with_win(1000));
        execute_test!(test, ExpectState, TCPSenderStateSummary::SynAcked);
        test.execute(&WriteBytes::new("abc".into(), false));
        test.execute(
            &ExpectSegment::new()
                .payload_size(3)
                .data("abc".into())
                .seqno(isn + 1),
        );
        test.execute(&Tick::new(rto - 5));
        test.execute(&WriteBytes::new("def".into(), false));
        test.execute(
            &ExpectSegment::new()
                .payload_size(3)
                .data("def".into())
                .seqno(isn + 4),
        );
        test.execute(&WriteBytes::new("ghi".into(), false));
        test.execute(
            &ExpectSegment::new()
                .payload_size(3)
                .data("ghi".into())
                .seqno(isn + 7),
        );
        test.execute(&AckReceived::new(WrappingInt32::new(isn + 1)).with_win(1000));
        test.execute(&Tick::new(6));
        test.execute(
            &ExpectSegment::new()
                .payload_size(3)
                .data("abc".into())
                .seqno(isn + 1),
        );
        execute_test!(test, ExpectNoSegment, true);
        test.execute(&Tick::new(rto * 2 - 5));
        execute_test!(test, ExpectNoSegment, true);
        test.execute(&Tick::new(5));
        test.execute(
            &ExpectSegment::new()
                .payload_size(3)
                .data("abc".into())
                .seqno(isn + 1),
        );
        execute_test!(test, ExpectNoSegment, true);
        test.execute(&Tick::new(rto * 4 - 5));
        execute_test!(test, ExpectNoSegment, true);
        test.execute(&AckReceived::new(WrappingInt32::new(isn + 4)).with_win(1000));
        test.execute(&Tick::new(rto - 1));
        execute_test!(test, ExpectNoSegment, true);
        test.execute(&Tick::new(2));
        test.execute(
            &ExpectSegment::new()
                .payload_size(3)
                .data("def".into())
                .seqno(isn + 4),
        );
        execute_test!(test, ExpectNoSegment, true);
    }
    {
        let isn: u32 = rng.gen_range(0..=std::u32::MAX);
        let mut cfg = TCPConfig::new();
        cfg.fixed_isn = Some(WrappingInt32::new(isn));

        let nicechars = "abcdefghijklmnopqrstuvwxyz";
        let bigstring: String = (0..DEFAULT_CAPACITY)
            .map(|_| {
                let idx = rng.gen_range(0..nicechars.len());
                nicechars.chars().nth(idx).unwrap()
            })
            .collect();

        let window_size: u16 = rng.gen_range(50000..=63000);

        let mut test = TCPSenderTestHarness::new("fill_window() correctly fills a big window", cfg);
        test.execute(&WriteBytes::new(bigstring.clone().into(), false));
        test.execute(
            &ExpectSegment::new()
                .no_flags()
                .syn(true)
                .payload_size(0)
                .seqno(isn),
        ); // first SYN
        test.execute(&AckReceived::new(WrappingInt32::new(isn + 1)).with_win(window_size));
        execute_test!(test, ExpectState, TCPSenderStateSummary::SynAcked);

        let mut i = 0;
        while (i + MAX_PAYLOAD_SIZE)
            < std::cmp::min(bigstring.as_bytes().len(), window_size as usize)
        {
            let expected_size = std::cmp::min(
                MAX_PAYLOAD_SIZE,
                std::cmp::min(bigstring.as_bytes().len(), window_size as usize) - i,
            );
            test.execute(
                &ExpectSegment::new()
                    .no_flags()
                    .payload_size(expected_size)
                    .data(bigstring[i..i + expected_size].into()) // Convert str to Vec<u8>
                    .seqno(isn + 1 + i as u32),
            );
            i += MAX_PAYLOAD_SIZE;
        }
    }
    {
        let isn: u32 = rng.gen_range(0..=std::u32::MAX);
        let rto: u64 = rng.gen_range(30..=10000);
        let mut cfg = TCPConfig::new();
        cfg.fixed_isn = Some(WrappingInt32::new(isn));
        cfg.rt_timeout = rto;

        let mut test =
            TCPSenderTestHarness::new("Retransmit a FIN-containing segment same as any other", cfg);
        test.execute(
            &ExpectSegment::new()
                .no_flags()
                .syn(true)
                .payload_size(0)
                .seqno(isn),
        );
        test.execute(&AckReceived::new(WrappingInt32::new(isn + 1)).with_win(1000));
        execute_test!(test, ExpectState, TCPSenderStateSummary::SynAcked);
        test.execute(&WriteBytes::new("abc".into(), true)); // with_end_input(true) indicates sending FIN
        test.execute(
            &ExpectSegment::new()
                .payload_size(3)
                .data("abc".into())
                .seqno(isn + 1)
                .fin(true),
        );
        test.execute(&Tick::new(rto - 1));
        execute_test!(test, ExpectNoSegment, true);
        test.execute(&Tick::new(2));
        test.execute(
            &ExpectSegment::new()
                .payload_size(3)
                .data("abc".into())
                .seqno(isn + 1)
                .fin(true),
        );
    }

    {
        let isn: u32 = rng.gen_range(0..=std::u32::MAX);
        let rto: u64 = rng.gen_range(30..=10000);
        let mut cfg = TCPConfig::new();
        cfg.fixed_isn = Some(WrappingInt32::new(isn));
        cfg.rt_timeout = rto;

        let mut test =
            TCPSenderTestHarness::new("Retransmit a FIN-only segment same as any other", cfg);
        test.execute(
            &ExpectSegment::new()
                .no_flags()
                .syn(true)
                .payload_size(0)
                .seqno(isn),
        );
        test.execute(&AckReceived::new(WrappingInt32::new(isn + 1)).with_win(1000));
        execute_test!(test, ExpectState, TCPSenderStateSummary::SynAcked);
        test.execute(&WriteBytes::new("abc".into(), false));
        test.execute(
            &ExpectSegment::new()
                .payload_size(3)
                .data("abc".into())
                .seqno(isn + 1)
                .no_flags(),
        );
        execute_test!(test, Close, true);
        test.execute(
            &ExpectSegment::new()
                .payload_size(0)
                .seqno(isn + 4)
                .fin(true),
        );
        test.execute(&Tick::new(rto - 1));
        execute_test!(test, ExpectNoSegment, true);
        test.execute(&AckReceived::new(WrappingInt32::new(isn + 4)).with_win(1000));
        test.execute(&Tick::new(rto - 1));
        execute_test!(test, ExpectNoSegment, true);
        test.execute(&Tick::new(2));
        test.execute(
            &ExpectSegment::new()
                .payload_size(0)
                .seqno(isn + 4)
                .fin(true),
        );
        test.execute(&Tick::new(2 * rto - 5));
        execute_test!(test, ExpectNoSegment, true);
        test.execute(&Tick::new(10));
        test.execute(
            &ExpectSegment::new()
                .payload_size(0)
                .seqno(isn + 4)
                .fin(true),
        );
    }
    {
        let isn: u32 = rng.gen_range(0..=std::u32::MAX);
        let rto: u64 = rng.gen_range(30..=10000);
        let mut cfg = TCPConfig::new();
        cfg.fixed_isn = Some(WrappingInt32::new(isn));
        cfg.rt_timeout = rto;

        let mut test = TCPSenderTestHarness::new(
            "Don't add FIN if this would make the segment exceed the receiver's window",
            cfg,
        );
        test.execute(
            &ExpectSegment::new()
                .no_flags()
                .syn(true)
                .payload_size(0)
                .seqno(isn),
        );
        test.execute(&WriteBytes::new("abc".into(), true)); // with_end_input(true) indicates sending FIN
        test.execute(&AckReceived::new(WrappingInt32::new(isn + 1)).with_win(3));
        execute_test!(test, ExpectState, TCPSenderStateSummary::SynAcked);
        test.execute(
            &ExpectSegment::new()
                .payload_size(3)
                .data("abc".into())
                .seqno(isn + 1)
                .no_flags(),
        );
        test.execute(&AckReceived::new(WrappingInt32::new(isn + 2)).with_win(2));
        execute_test!(test, ExpectNoSegment, true);
        test.execute(&AckReceived::new(WrappingInt32::new(isn + 3)).with_win(1));
        execute_test!(test, ExpectNoSegment, true);
        test.execute(&AckReceived::new(WrappingInt32::new(isn + 4)).with_win(1));
        test.execute(
            &ExpectSegment::new()
                .payload_size(0)
                .seqno(isn + 4)
                .fin(true),
        );
    }
    {
        let mut rng = rand::thread_rng();
        let isn: u32 = rng.gen_range(0..=std::u32::MAX);
        let rto: u64 = rng.gen_range(30..=10000);
        let mut cfg = TCPConfig::new();
        cfg.fixed_isn = Some(WrappingInt32::new(isn));
        cfg.rt_timeout = rto;

        let mut test =
            TCPSenderTestHarness::new("Don't send FIN by itself if the window is full", cfg);
        test.execute(
            &ExpectSegment::new()
                .no_flags()
                .syn(true)
                .payload_size(0)
                .seqno(isn),
        );
        test.execute(&WriteBytes::new("abc".into(), false));
        execute_test!(test, ExpectNoSegment, true);
        test.execute(&AckReceived::new(WrappingInt32::new(isn + 1)).with_win(3));
        execute_test!(test, ExpectState, TCPSenderStateSummary::SynAcked);
        test.execute(
            &ExpectSegment::new()
                .payload_size(3)
                .data("abc".into())
                .seqno(isn + 1)
                .no_flags(),
        );
        execute_test!(test, Close, true);
        execute_test!(test, ExpectNoSegment, true);
        test.execute(&AckReceived::new(WrappingInt32::new(isn + 2)).with_win(2));
        execute_test!(test, ExpectNoSegment, true);
        test.execute(&AckReceived::new(WrappingInt32::new(isn + 3)).with_win(1));
        execute_test!(test, ExpectNoSegment, true);
        test.execute(&AckReceived::new(WrappingInt32::new(isn + 4)).with_win(1));
        test.execute(
            &ExpectSegment::new()
                .payload_size(0)
                .seqno(isn + 4)
                .fin(true),
        );
    }
}

#[test]
fn test2() {
    let mut rng = rand::thread_rng();
    {
        let isn: u32 = rng.gen_range(0..=std::u32::MAX);
        let rto: u64 = rng.gen_range(30..=10000);
        let mut cfg = TCPConfig::new();
        cfg.fixed_isn = Some(WrappingInt32::new(isn));
        cfg.rt_timeout = rto;

        let nicechars = "abcdefghijklmnopqrstuvwxyz";
        let bigstring: String = (0..MAX_PAYLOAD_SIZE)
            .map(|_| {
                let idx = rng.gen_range(0..nicechars.len());
                nicechars.chars().nth(idx).unwrap()
            })
            .collect();

        let mut test = TCPSenderTestHarness::new("MAX_PAYLOAD_SIZE limits payload only", cfg);
        test.execute(
            &ExpectSegment::new()
                .no_flags()
                .syn(true)
                .payload_size(0)
                .seqno(isn),
        );
        test.execute(&WriteBytes::new(bigstring.clone().into(), true)); // with_end_input(true) indicates sending FIN
        test.execute(&AckReceived::new(WrappingInt32::new(isn + 1)).with_win(40000));
        test.execute(
            &ExpectSegment::new()
                .payload_size(MAX_PAYLOAD_SIZE)
                .data(bigstring.into())
                .seqno(isn + 1)
                .fin(true),
        );
        execute_test!(test, ExpectState, TCPSenderStateSummary::FinSent);
        execute_test!(
            test,
            AckReceived,
            WrappingInt32::new(isn + 2 + MAX_PAYLOAD_SIZE as u32)
        );
        execute_test!(test, ExpectState, TCPSenderStateSummary::FinAcked);
    }
    {
        let isn: u32 = rng.gen_range(0..=std::u32::MAX);
        let rto: u64 = rng.gen_range(30..=10000);
        let mut cfg = TCPConfig::new();
        cfg.fixed_isn = Some(WrappingInt32::new(isn));
        cfg.rt_timeout = rto;

        let mut test = TCPSenderTestHarness::new(
            "When filling window, treat a '0' window size as equal to '1' but don't back off RTO",
            cfg,
        );
        test.execute(
            &ExpectSegment::new()
                .no_flags()
                .syn(true)
                .payload_size(0)
                .seqno(isn),
        );
        test.execute(&WriteBytes::new("abc".into(), false));
        execute_test!(test, ExpectNoSegment, true);
        test.execute(&AckReceived::new(WrappingInt32::new(isn + 1)).with_win(0));

        execute_test!(test, ExpectState, TCPSenderStateSummary::SynAcked);
        test.execute(
            &ExpectSegment::new()
                .payload_size(1)
                .data("a".into())
                .seqno(isn + 1)
                .no_flags(),
        );
        test.execute(&Close {});
        execute_test!(test, ExpectNoSegment, true);

        for _ in 0..5 {
            test.execute(&Tick::new(rto - 1));
            execute_test!(test, ExpectNoSegment, true);
            test.execute(&Tick::new(1));
            test.execute(
                &ExpectSegment::new()
                    .payload_size(1)
                    .data("a".into())
                    .seqno(isn + 1)
                    .no_flags(),
            );
        }

        test.execute(&AckReceived::new(WrappingInt32::new(isn + 2)).with_win(0));
        test.execute(
            &ExpectSegment::new()
                .payload_size(1)
                .data("b".into())
                .seqno(isn + 2)
                .no_flags(),
        );

        for _ in 0..5 {
            test.execute(&Tick::new(rto - 1));
            execute_test!(test, ExpectNoSegment, true);
            test.execute(&Tick::new(1));
            test.execute(
                &ExpectSegment::new()
                    .payload_size(1)
                    .data("b".into())
                    .seqno(isn + 2)
                    .no_flags(),
            );
        }
        test.execute(&AckReceived::new(WrappingInt32::new(isn + 3)).with_win(0));
        test.execute(
            &ExpectSegment::new()
                .payload_size(1)
                .data("c".into())
                .seqno(isn + 3)
                .no_flags(),
        );

        for _ in 0..5 {
            test.execute(&Tick::new(rto - 1));
            execute_test!(test, ExpectNoSegment, true);
            test.execute(&Tick::new(1));
            test.execute(
                &ExpectSegment::new()
                    .payload_size(1)
                    .data("c".into())
                    .seqno(isn + 3)
                    .no_flags(),
            );
        }

        test.execute(&AckReceived::new(WrappingInt32::new(isn + 4)).with_win(0));
        test.execute(
            &ExpectSegment::new()
                .payload_size(0)
                .data("".into())
                .seqno(isn + 4)
                .fin(true),
        );

        for _ in 0..5 {
            test.execute(&Tick::new(rto - 1));
            execute_test!(test, ExpectNoSegment, true);
            test.execute(&Tick::new(1));
            test.execute(
                &ExpectSegment::new()
                    .payload_size(0)
                    .data("".into())
                    .seqno(isn + 4)
                    .fin(true),
            );
        }
    }
    {
        let isn: u32 = rng.gen_range(0..=std::u32::MAX);
        let rto: u64 = rng.gen_range(30..=10000);
        let mut cfg = TCPConfig::new();
        cfg.fixed_isn = Some(WrappingInt32::new(isn));
        cfg.rt_timeout = rto;

        let mut test = TCPSenderTestHarness::new(
            "Unlike a zero-size window, a full window of nonzero size should be respected",
            cfg,
        );
        test.execute(
            &ExpectSegment::new()
                .no_flags()
                .syn(true)
                .payload_size(0)
                .seqno(isn),
        );
        test.execute(&WriteBytes::new("abc".into(), false));
        execute_test!(test, ExpectNoSegment, true);
        test.execute(&AckReceived::new(WrappingInt32::new(isn + 1)).with_win(1));
        execute_test!(test, ExpectState, TCPSenderStateSummary::SynAcked);
        test.execute(
            &ExpectSegment::new()
                .payload_size(1)
                .data("a".into())
                .seqno(isn + 1)
                .no_flags(),
        );
        test.execute(&Tick::new(rto - 1));
        execute_test!(test, ExpectNoSegment, true);
        test.execute(&Tick::new(1));
        test.execute(
            &ExpectSegment::new()
                .payload_size(1)
                .data("a".into())
                .seqno(isn + 1)
                .no_flags(),
        );

        test.execute(&Close {});
        test.execute(&Tick::new(2 * rto - 1));
        execute_test!(test, ExpectNoSegment, true);
        test.execute(&Tick::new(1));
        test.execute(
            &ExpectSegment::new()
                .payload_size(1)
                .data("a".into())
                .seqno(isn + 1)
                .no_flags(),
        );

        test.execute(&Tick::new(4 * rto - 1));
        execute_test!(test, ExpectNoSegment, true);
        test.execute(&Tick::new(1));
        test.execute(
            &ExpectSegment::new()
                .payload_size(1)
                .data("a".into())
                .seqno(isn + 1)
                .no_flags(),
        );

        test.execute(&AckReceived::new(WrappingInt32::new(isn + 2)).with_win(3));
        test.execute(
            &ExpectSegment::new()
                .payload_size(2)
                .data("bc".into())
                .seqno(isn + 2)
                .fin(true),
        );
    }
    {
        let isn: u32 = rng.gen_range(0..=std::u32::MAX);
        let rto: u64 = rng.gen_range(30..=10000);
        let mut cfg = TCPConfig::new();
        cfg.fixed_isn = Some(WrappingInt32::new(isn));
        cfg.rt_timeout = rto;

        let mut test =
            TCPSenderTestHarness::new("Repeated ACKs and outdated ACKs are harmless", cfg);
        test.execute(
            &ExpectSegment::new()
                .no_flags()
                .syn(true)
                .payload_size(0)
                .seqno(isn),
        );
        test.execute(&AckReceived::new(WrappingInt32::new(isn + 1)).with_win(1000));
        execute_test!(test, ExpectState, TCPSenderStateSummary::SynAcked);
        test.execute(&WriteBytes::new("abcdefg".into(), false));
        test.execute(
            &ExpectSegment::new()
                .payload_size(7)
                .data("abcdefg".into())
                .seqno(isn + 1),
        );
        test.execute(&AckReceived::new(WrappingInt32::new(isn + 8)).with_win(1000));
        execute_test!(test, ExpectNoSegment, true);
        // Repeated ACKs
        for _ in 0..3 {
            test.execute(&AckReceived::new(WrappingInt32::new(isn + 8)).with_win(1000));
        }
        execute_test!(test, ExpectNoSegment, true);
        // Outdated ACKs
        for _ in 0..3 {
            test.execute(&AckReceived::new(WrappingInt32::new(isn + 1)).with_win(1000));
        }
        execute_test!(test, ExpectNoSegment, true);
        test.execute(&WriteBytes::new("ijkl".into(), true)); // with_end_input(true) indicates sending FIN
        test.execute(
            &ExpectSegment::new()
                .payload_size(4)
                .data("ijkl".into())
                .seqno(isn + 8)
                .fin(true),
        );
        // More repeated ACKs at different sequence numbers
        for _ in 0..3 {
            test.execute(&AckReceived::new(WrappingInt32::new(isn + 1)).with_win(1000));
        }
        for _ in 0..3 {
            test.execute(&AckReceived::new(WrappingInt32::new(isn + 8)).with_win(1000));
        }
        for _ in 0..3 {
            test.execute(&AckReceived::new(WrappingInt32::new(isn + 12)).with_win(1000));
        }
        test.execute(&AckReceived::new(WrappingInt32::new(isn + 1)).with_win(1000));
        test.execute(&Tick::new(5 * rto));
        test.execute(
            &ExpectSegment::new()
                .payload_size(4)
                .data("ijkl".into())
                .seqno(isn + 8)
                .fin(true),
        );
        execute_test!(test, ExpectNoSegment, true);
        // Final state check
        test.execute(&AckReceived::new(WrappingInt32::new(isn + 13)).with_win(1000));
        test.execute(&AckReceived::new(WrappingInt32::new(isn + 1)).with_win(1000));
        test.execute(&Tick::new(5 * rto));
        execute_test!(test, ExpectNoSegment, true);
        execute_test!(test, ExpectState, TCPSenderStateSummary::FinAcked);
    }
}
