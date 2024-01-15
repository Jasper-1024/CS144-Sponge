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
            offset,
            "test 1 - number of bytes RX is incorrect"
        );
        assert_eq!(
            result.as_slice(),
            d.as_slice(),
            "test 1 - content of RX bytes is incorrect"
        );
    }
}
