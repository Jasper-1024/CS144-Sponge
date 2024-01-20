mod fsm_stream_reassembler_harness;
use fsm_stream_reassembler_harness::*;

#[test]
fn test1() {
    let mut test = ReassemblerTestHarness::new(65000);

    test.submit_segment(b"abcd", 0);
    test.bytes_assembled(4);
    test.bytes_available(b"abcd");
    test.not_at_eof();

    test.submit_segment(b"abcd", 0);
    test.bytes_assembled(4);
    test.bytes_available(b"");
    test.not_at_eof();
}

#[test]
fn test2() {
    let mut test = ReassemblerTestHarness::new(65000);

    test.submit_segment(b"abcd", 0);
    test.bytes_assembled(4);
    test.bytes_available(b"abcd");
    test.not_at_eof();

    test.submit_segment(b"abcd", 4);
    test.bytes_assembled(8);
    test.bytes_available(b"abcd");
    test.not_at_eof();

    test.submit_segment(b"abcd", 0);
    test.bytes_assembled(8);
    test.bytes_available(b"");
    test.not_at_eof();

    test.submit_segment(b"abcd", 4);
    test.bytes_assembled(8);
    test.bytes_available(b"");
    test.not_at_eof();
}

#[test]
fn test3() {
    let mut test = ReassemblerTestHarness::new(65000);

    test.submit_segment(b"abcdefgh", 0);
    test.bytes_assembled(8);
    test.bytes_available(b"abcdefgh");
    test.not_at_eof();

    let data = b"abcdefgh";

    for i in 0..1000 {
        let start_i = rand::random::<usize>() % 8;
        let start = &data[start_i..];

        let end_i = rand::random::<usize>() % (8 - start_i) + start_i;
        let end = &data[..end_i];

        test.submit_segment(start, start_i as u64);
        test.bytes_assembled(8);
        test.bytes_available(b"");
        test.not_at_eof();
    }
}

#[test]
fn test4() {
    let mut test = ReassemblerTestHarness::new(65000);

    test.submit_segment(b"abcd", 0);
    test.bytes_assembled(4);
    test.bytes_available(b"abcd");
    test.not_at_eof();

    test.submit_segment(b"abcdef", 0);
    test.bytes_assembled(6);
    test.bytes_available(b"ef");
    test.not_at_eof();
}
