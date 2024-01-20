mod fsm_stream_reassembler_harness;
use fsm_stream_reassembler_harness::*;

#[test]
fn test1() {
    let mut test = ReassemblerTestHarness::new(1000);

    test.submit_segment(b"a", 0);
    test.submit_segment(b"ab", 0);

    test.bytes_assembled(2);
    test.bytes_available(b"ab");
}

#[test]
fn test2() {
    let mut test = ReassemblerTestHarness::new(1000);

    test.submit_segment(b"a", 0);
    test.bytes_available(b"a");

    test.submit_segment(b"ab", 0);
    test.bytes_available(b"b");
    test.bytes_assembled(2);
}

#[test]
fn test3() {
    let mut test = ReassemblerTestHarness::new(1000);

    test.submit_segment(b"b", 1);
    test.bytes_available(b"");

    test.submit_segment(b"ab", 0);
    test.bytes_available(b"ab");
    test.unassembled_bytes(0);
    test.bytes_assembled(2);
}

#[test]
fn test4() {
    let mut test = ReassemblerTestHarness::new(1000);

    test.submit_segment(b"b", 1);
    test.bytes_available(b"");

    test.submit_segment(b"bc", 1);
    test.bytes_available(b"");
    test.unassembled_bytes(2);
    test.bytes_assembled(0);
}

#[test]
fn test5() {
    let mut test = ReassemblerTestHarness::new(1000);

    test.submit_segment(b"c", 2);
    test.bytes_available(b"");

    test.submit_segment(b"bcd", 1);
    test.bytes_available(b"");
    test.unassembled_bytes(3);
    test.bytes_assembled(0);
}

#[test]
fn test6() {
    let mut test = ReassemblerTestHarness::new(1000);

    test.submit_segment(b"b", 1);
    test.submit_segment(b"d", 3);
    test.bytes_available(b"");

    test.submit_segment(b"bcde", 1);
    test.bytes_available(b"");
    test.bytes_assembled(0);
    test.unassembled_bytes(4);
}

#[test]
fn test7() {
    let mut test = ReassemblerTestHarness::new(1000);

    test.submit_segment(b"c", 2);
    test.submit_segment(b"bcd", 1);

    test.bytes_available(b"");
    test.bytes_assembled(0);
    test.unassembled_bytes(3);

    test.submit_segment(b"a", 0);
    test.bytes_available(b"abcd");
    test.bytes_assembled(4);
    test.unassembled_bytes(0);
}

#[test]
fn test8() {
    let mut test = ReassemblerTestHarness::new(1000);

    test.submit_segment(b"bcd", 1);
    test.submit_segment(b"c", 2);

    test.bytes_available(b"");
    test.bytes_assembled(0);
    test.unassembled_bytes(3);

    test.submit_segment(b"a", 0);
    test.bytes_available(b"abcd");
    test.bytes_assembled(4);
    test.unassembled_bytes(0);
}
// Additional tests converted
