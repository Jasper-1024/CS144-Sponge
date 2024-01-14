mod fsm_stream_reassembler_harness;
use fsm_stream_reassembler_harness::*;

#[test]
fn test1() {
    let mut test = ReassemblerTestHarness::new(65000);

    test.submit_segment(b"abcd", 0);
    test.bytes_assembled(4);
    test.bytes_available(b"abcd");
    test.not_at_eof();

    test.submit_segment(b"efgh", 4);
    test.bytes_assembled(8);
    test.bytes_available(b"efgh");
    test.not_at_eof();
}

#[test]
fn test2() {
    let mut test = ReassemblerTestHarness::new(65000);

    test.submit_segment(b"abcd", 0);
    test.bytes_assembled(4);
    test.not_at_eof();

    test.submit_segment(b"efgh", 4);
    test.bytes_assembled(8);

    test.bytes_available(b"abcdefgh");
    test.not_at_eof();
}

#[test]
fn test3() {
    let mut test = ReassemblerTestHarness::new(65000);
    let mut ss = String::new();

    for i in 0..100 {
        test.bytes_assembled(4 * i);
        test.submit_segment(b"abcd", 4 * i as u64);
        test.not_at_eof();

        ss.push_str("abcd");
    }

    test.bytes_available(ss.as_bytes());
    test.not_at_eof();
}

#[test]
fn test4() {
    let mut test = ReassemblerTestHarness::new(65000);

    for i in 0..100 {
        test.bytes_assembled(4 * i);
        test.submit_segment(b"abcd", 4 * i as u64);
        test.not_at_eof();

        test.bytes_available(b"abcd");
    }
}
