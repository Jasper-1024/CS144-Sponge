use cs144_rust::byte_stream::{ByteStream, ByteStreamTrait};
use rand::{thread_rng, Rng};

#[test]
fn many_writes() {
    let mut rng = thread_rng();
    const NREPS: usize = 1000;
    const MIN_WRITE: usize = 10;
    const MAX_WRITE: usize = 200;
    const CAPACITY: usize = MAX_WRITE * NREPS;

    let mut stream = ByteStream::new(CAPACITY);
    let mut acc: usize = 0;

    for _ in 0..NREPS {
        let size: usize = MIN_WRITE + (rng.gen_range(0..(MAX_WRITE - MIN_WRITE)));
        let data: Vec<u8> = (0..size)
            .map(|_| b'a' + rng.gen_range(0..26) as u8)
            .collect();

        assert_eq!(stream.write(&data), size);
        acc += size;

        assert_eq!(stream.input_ended(), false);
        assert_eq!(stream.buffer_empty(), false);
        assert_eq!(stream.eof(), false);
        assert_eq!(stream.bytes_read(), 0);
        assert_eq!(stream.bytes_written(), acc as u64);
        assert_eq!(stream.remaining_capacity(), CAPACITY - acc);
        assert_eq!(stream.buffer_size(), acc);
    }
}
