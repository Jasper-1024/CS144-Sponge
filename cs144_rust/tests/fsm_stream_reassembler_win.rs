mod fsm_stream_reassembler_harness;

use cs144_rust::{
    byte_stream::{ByteStreamTrait},
    stream_reassembler::{StreamReassembler, StreamReassemblerTrait},
};
use rand::{seq::SliceRandom, thread_rng, Rng};

#[allow(dead_code)]
const NREPS: usize = 32/* ... */; // You should define NREPS
#[allow(dead_code)]
const NSEGS: usize = 128/* ... */; // You should define NSEGS
#[allow(dead_code)]
const MAX_SEG_LEN: usize = 2048/* ... */; // You should define MAX_SEG_LEN

fn read(reassembler: &StreamReassembler) -> Vec<u8> {
    let binding = reassembler.stream_out();
    let buffer_size = binding.borrow().buffer_size();
    return binding.borrow_mut().read(buffer_size);
}

#[test]
fn test_overlapping_segments() {
    let mut rd = thread_rng();

    for _ in 0..NREPS {
        let mut buf = StreamReassembler::new(NSEGS * MAX_SEG_LEN);

        let mut seq_size = Vec::new();
        let mut offset = 0;
        for _ in 0..NSEGS {
            let size = 1 + (rd.gen_range(0..MAX_SEG_LEN - 1)); // 0..MAX_SEG_LEN-1 内的随机数| 长度
            let offs = std::cmp::min(offset, 1 + (rd.gen_range(0..1023) as usize)); // 0..1023 内的随机数| 偏移
            seq_size.push((offset - offs, size + offs));
            offset += size;
        }
        seq_size.shuffle(&mut rd); // seq_size 随机打乱

        let mut d: Vec<u8> = vec![0; offset];
        d.iter_mut().for_each(|x| *x = rd.gen()); // d 随机填充

        for &(off, sz) in &seq_size {
            let dd: &[u8] = &d[off..off + sz];
            buf.push_substring(dd, off as u64, off + sz == offset);
        }

        let result = read(&buf);

        assert_eq!(
            buf.stream_out().borrow().bytes_written(),
            offset,
            "test_overlapping_segments - number of RX bytes is incorrect"
        );
        assert_eq!(
            result.as_slice(),
            d.as_slice(),
            "test_overlapping_segments - content of RX bytes is incorrect"
        );
    }
}
