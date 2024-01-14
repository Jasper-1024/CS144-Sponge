mod fsm_stream_reassembler_harness;
use fsm_stream_reassembler_harness::*;

#[test]
fn test1() {
    let mut test = ReassemblerTestHarness::new(65000);

    test.submit_segment(b"b", 1);

    test.bytes_assembled(0);
    test.bytes_available(b"");
    test.not_at_eof();
}

#[test]
fn test2() {
    let mut test = ReassemblerTestHarness::new(65000);

    test.submit_segment(b"b", 1);
    test.submit_segment(b"a", 0);

    test.bytes_assembled(2);
    test.bytes_available(b"ab");
    test.not_at_eof();
}

#[test]
fn test3() {
    let mut test = ReassemblerTestHarness::new(65000);

    test.submit_segment_default(b"b", 1, true);

    test.bytes_assembled(0);
    test.bytes_available(b"");
    test.not_at_eof();

    test.submit_segment(b"a", 0);

    test.bytes_assembled(2);
    test.bytes_available(b"ab");
    test.at_eof();
}

#[test]
fn test4() {
    let mut test = ReassemblerTestHarness::new(65000);

    test.submit_segment(b"b", 1);
    test.submit_segment(b"ab", 0);

    test.bytes_assembled(2);
    test.bytes_available(b"ab");
    test.not_at_eof();
}

#[test]
fn test5() {
    let mut test = ReassemblerTestHarness::new(65000);

    test.submit_segment(b"b", 1);
    test.bytes_assembled(0);
    test.bytes_available(b"");
    test.not_at_eof();

    test.submit_segment(b"d", 3);
    test.bytes_assembled(0);
    test.bytes_available(b"");
    test.not_at_eof();

    test.submit_segment(b"c", 2);
    test.bytes_assembled(0);
    test.bytes_available(b"");
    test.not_at_eof();

    test.submit_segment(b"a", 0);

    test.bytes_assembled(4);
    test.bytes_available(b"abcd");
    test.not_at_eof();
}

#[test]
fn test6() {
    let mut test = ReassemblerTestHarness::new(65000);

    test.submit_segment(b"b", 1);
    test.bytes_assembled(0);
    test.bytes_available(b"");
    test.not_at_eof();

    test.submit_segment(b"d", 3);
    test.bytes_assembled(0);
    test.bytes_available(b"");
    test.not_at_eof();

    test.submit_segment(b"abc", 0);

    test.bytes_assembled(4);
    test.bytes_available(b"abcd");
    test.not_at_eof();
}

#[test]
fn test7() {
    let mut test = ReassemblerTestHarness::new(65000);

    test.submit_segment(b"b", 1);
    test.bytes_assembled(0);
    test.bytes_available(b"");
    test.not_at_eof();

    test.submit_segment(b"d", 3);
    test.bytes_assembled(0);
    test.bytes_available(b"");
    test.not_at_eof();

    test.submit_segment(b"a", 0);
    test.bytes_assembled(2);
    test.bytes_available(b"ab");
    test.not_at_eof();

    test.submit_segment(b"c", 2);
    test.bytes_assembled(4);
    test.bytes_available(b"cd");
    test.not_at_eof();

    test.submit_segment_default(b"", 4, true);
    test.bytes_assembled(4);
    test.bytes_available(b"");
}
