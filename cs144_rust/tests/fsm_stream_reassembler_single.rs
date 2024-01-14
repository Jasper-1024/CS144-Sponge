mod fsm_stream_reassembler_harness;
use fsm_stream_reassembler_harness::*;

#[test]
fn test1() {
    let mut test = ReassemblerTestHarness::new(65000);

    test.bytes_assembled(0);
    test.bytes_available(b"");
    test.not_at_eof();
}

#[test]
fn test2() {
    let mut test = ReassemblerTestHarness::new(65000);

    test.submit_segment(b"a", 0);

    test.bytes_assembled(1);
    test.bytes_available(b"a");
    test.not_at_eof();
}

#[test]
fn test3() {
    let mut test = ReassemblerTestHarness::new(65000);

    test.submit_segment_default(b"a", 0, true);

    test.bytes_assembled(1);
    test.bytes_available(b"a");
}

#[test]
fn test4() {
    let mut test = ReassemblerTestHarness::new(65000);

    test.submit_segment_default(b"", 0, true);

    test.bytes_assembled(0);
    test.bytes_available(b"");
}

#[test]
fn test5() {
    let mut test = ReassemblerTestHarness::new(65000);

    test.submit_segment_default(b"b", 0, true);

    test.bytes_assembled(1);
    test.bytes_available(b"b");
}

#[test]
fn test6() {
    let mut test = ReassemblerTestHarness::new(65000);

    test.submit_segment(b"", 0);

    test.bytes_assembled(0);
    test.bytes_available(b"");
    test.not_at_eof();
}

#[test]
fn test7() {
    let mut test = ReassemblerTestHarness::new(8);

    test.submit_segment(b"abcdefgh", 0);

    test.bytes_assembled(8);
    test.bytes_available(b"abcdefgh");
    test.not_at_eof();
}

#[test]
fn test8() {
    let mut test = ReassemblerTestHarness::new(8);

    test.submit_segment_default(b"abcdefgh", 0, true);

    test.bytes_assembled(8);
    test.bytes_available(b"abcdefgh");
}

#[test]
fn test9() {
    let mut test = ReassemblerTestHarness::new(8);

    test.submit_segment(b"abc", 0);
    test.bytes_assembled(3);

    test.submit_segment_default(b"bcdefgh", 1, true);

    test.bytes_assembled(8);
    test.bytes_available(b"abcdefgh");
}

#[test]
fn test10() {
    let mut test = ReassemblerTestHarness::new(8);

    test.submit_segment(b"abc", 0);
    test.bytes_assembled(3);
    test.not_at_eof();

    test.submit_segment_default(b"ghX", 6, true);
    test.bytes_assembled(3);
    test.not_at_eof();

    test.submit_segment(b"cdefg", 2);
    test.bytes_assembled(8);
    test.bytes_available(b"abcdefgh");
    test.not_at_eof();
}

#[test]
fn test11() {
    let mut test = ReassemblerTestHarness::new(8);

    test.submit_segment(b"abc", 0);
    test.bytes_assembled(3);
    test.not_at_eof();

    // Ignore empty segment
    test.submit_segment(b"", 6);
    test.bytes_assembled(3);
    test.not_at_eof();

    test.submit_segment_default(b"de", 3, true);
    test.bytes_assembled(5);
    test.bytes_available(b"abcde");
}
