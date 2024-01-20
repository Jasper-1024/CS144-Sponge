mod fsm_stream_reassembler_harness;
use fsm_stream_reassembler_harness::*;

#[test]
fn test1() {
    let mut test = ReassemblerTestHarness::new(2);

    test.submit_segment(b"ab", 0);
    test.bytes_assembled(2);
    test.bytes_available(b"ab");

    test.submit_segment(b"cd", 2);
    test.bytes_assembled(4);
    test.bytes_available(b"cd");

    test.submit_segment(b"ef", 4);
    test.bytes_assembled(6);
    test.bytes_available(b"ef");
}

#[test]
fn test2() {
    let mut test = ReassemblerTestHarness::new(2);

    test.submit_segment(b"ab", 0);
    test.bytes_assembled(2);

    test.submit_segment(b"cd", 2);
    test.bytes_assembled(2);

    test.bytes_available(b"ab");
    test.bytes_assembled(2);

    test.submit_segment(b"cd", 2);
    test.bytes_assembled(4);

    test.bytes_available(b"cd");
}

#[test]
fn test3() {
    let mut test = ReassemblerTestHarness::new(2);

    test.submit_segment(b"bX", 1);
    test.bytes_assembled(0);

    test.submit_segment(b"a", 0);
    test.bytes_assembled(2);

    test.bytes_available(b"ab");
}

#[test]
fn test4() {
    let mut test = ReassemblerTestHarness::new(1);

    test.submit_segment(b"ab", 0);
    test.bytes_assembled(1);

    test.submit_segment(b"ab", 0);
    test.bytes_assembled(1);

    test.bytes_available(b"a");
    test.bytes_assembled(1);

    test.submit_segment(b"abc", 0);
    test.bytes_assembled(2);

    test.bytes_available(b"b");
    test.bytes_assembled(2);
}

#[test]
fn test5() {
    let mut test = ReassemblerTestHarness::new(8);

    test.submit_segment(b"a", 0);
    test.bytes_assembled(1);
    test.bytes_available(b"a");

    test.submit_segment(b"bc", 1);
    test.bytes_assembled(3);

    test.submit_segment_default(b"ghi", 6, true);
    test.bytes_assembled(3);

    test.submit_segment(b"cdefg", 2);
    test.bytes_assembled(9);
    test.bytes_available(b"bcdefghi");
}

#[test]
fn test6() {
    let mut data_segments: Vec<Vec<u8>> = Vec::new();
    let mut test = ReassemblerTestHarness::new(3);

    // 在循环外预分配足够大的内存来存储所有的数据段
    for i in (0..99997).step_by(3) {
        data_segments.push(vec![
            i as u8,
            (i + 1) as u8,
            (i + 2) as u8,
            (i + 13) as u8,
            (i + 47) as u8,
            (i + 9) as u8,
        ]);
    }

    // 迭代存储的数据段, 没办法这个只能这样处理 😶‍🌫️😶‍🌫️
    for (i, segment) in data_segments.iter().enumerate() {
        let index = (i * 3) as u64; // 每个segment的起始索引
        test.submit_segment(segment, index);
        test.bytes_assembled((index + 3).try_into().unwrap());
        test.bytes_available(&segment[0..3]);
    }
}
