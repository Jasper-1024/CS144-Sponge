mod fsm_stream_reassembler_harness;

use cs144_rust::{
    byte_stream::{self, ByteStreamTrait},
    stream_reassembler::{StreamReassembler, StreamReassemblerTrait},
};
use rand::{seq::SliceRandom, thread_rng, Rng};

#[allow(dead_code)]
const NREPS: usize = 32/* ... */; // You should define NREPS
#[allow(dead_code)]
const MAX_SEG_LEN: usize = 2048/* ... */; // You should define MAX_SEG_LEN
#[allow(dead_code)]
const NSEGS: usize = 128/* ... */; // You should define NSEGS

fn read(reassembler: &StreamReassembler) -> Vec<u8> {
    let binding = reassembler.stream_out();
    let buffer_size = binding.borrow().buffer_size();
    return binding.borrow_mut().read(buffer_size);
}

#[test]
fn test1() {
    let mut rng = thread_rng();
    for _ in 0..NREPS {
        let mut buf = StreamReassembler::new(MAX_SEG_LEN * NSEGS);
        let mut seq_size = Vec::new();
        let mut offset = 0;
        for _ in 0..NSEGS {
            let size = 1 + rng.gen_range(0..MAX_SEG_LEN - 1);
            seq_size.push((offset, size));
            offset += size;
        }
        seq_size.shuffle(&mut rng);

        let mut d = vec![0u8; offset];
        rng.fill(&mut d[..]);

        for &(off, sz) in &seq_size {
            let dd = &d[off..off + sz];
            buf.push_substring(dd, off as u64, off + sz == offset);
        }

        let result = read(&buf);
        assert_eq!(
            buf.stream_out().borrow().bytes_written(),
            offset as u64,
            "test 1 - number of bytes RX is incorrect"
        );
        assert_eq!(
            result.as_slice(),
            d.as_slice(),
            "test 1 - content of RX bytes is incorrect"
        );
    }
}
#[test]
fn test3() {
    let mut rng = thread_rng();

    for _ in 0..NREPS {
        let mut buf = StreamReassembler::new(65_000);

        let size = 1024;
        let mut d = vec![0u8; size];
        rng.fill(&mut d[..]);

        buf.push_substring(&d, 0, false);
        buf.push_substring(&d[10..], size as u64 + 10, false);

        let res1 = read(&buf);
        assert_eq!(
            buf.stream_out().borrow().bytes_written(),
            size as u64,
            "test 3 - number of RX bytes is incorrect"
        );
        assert_eq!(
            res1.as_slice(),
            d.as_slice(),
            "test 3 - content of RX bytes is incorrect"
        );

        buf.push_substring(&d[..7], size as u64, false);
        buf.push_substring(&d[7..8], size as u64 + 7, true);

        let res2 = read(&buf);
        assert_eq!(
            buf.stream_out().borrow().bytes_written(),
            size as u64 + 8,
            "test 3 - number of RX bytes is incorrect after 2nd read"
        );
        assert_eq!(
            res2.as_slice(),
            d[..8].to_vec().as_slice(),
            "test 3 - content of RX bytes is incorrect after 2nd read"
        ); // 这里和 c++ 测试不一样, c++ 中对应比较是 比较d中 与 res2 相同长度的部分, res2 没有的就终止了..
           // rust 要求长度完全一致, 这里只能传入 d[..8].to_vec().as_slice()
    }
}

#[test]
fn test4() {
    let mut rng = thread_rng();

    for _ in 0..NREPS {
        let mut buf = StreamReassembler::new(65_000);

        let size = 1024;
        let mut d = vec![0u8; size];
        rng.fill(&mut d[..]);

        buf.push_substring(&d, 0, false);
        buf.push_substring(&d[10..], size as u64 + 10, false);

        let res1 = read(&buf);
        assert_eq!(
            buf.stream_out().borrow().bytes_written(),
            size as u64,
            "test 4 - number of RX bytes is incorrect"
        );
        assert_eq!(
            res1.as_slice(),
            d.as_slice(),
            "test 4 - content of RX bytes is incorrect"
        );

        buf.push_substring(&d[..15], size as u64, true);

        let res2 = read(&buf);
        let bytes_written = buf.stream_out().borrow().bytes_written();
        assert!(
            bytes_written == 2 * size as u64 || bytes_written == size as u64 + 15,
            "test 4 - number of RX bytes is incorrect after 2nd read"
        );
        assert_eq!(
            res2.as_slice(),
            d.as_slice(),
            "test 4 - content of RX bytes is incorrect after 2nd read"
        );
    }
}
