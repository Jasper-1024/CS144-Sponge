use cs144_rust::byte_stream::{ByteStream, ByteStreamTrait};
#[test]
fn overwrite() {
    let mut stream = ByteStream::new(2);

    assert_eq!(stream.write(b"cat"), 2);
    assert_eq!(stream.input_ended(), false);
    assert_eq!(stream.buffer_empty(), false);
    assert_eq!(stream.eof(), false);
    assert_eq!(stream.bytes_read(), 0);
    assert_eq!(stream.bytes_written(), 2); // 已写入的字节数
    assert_eq!(stream.remaining_capacity(), 0);
    assert_eq!(stream.buffer_size(), 2);
    assert_eq!(stream.peek_output(2), b"ca");

    assert_eq!(stream.write(b"t"), 0);
    assert_eq!(stream.input_ended(), false);
    assert_eq!(stream.buffer_empty(), false);
    assert_eq!(stream.eof(), false);
    assert_eq!(stream.bytes_read(), 0);
    assert_eq!(stream.bytes_written(), 2);
    assert_eq!(stream.remaining_capacity(), 0);
    assert_eq!(stream.buffer_size(), 2);
    assert_eq!(stream.peek_output(2), b"ca");
}
#[test]
fn overwrite_clear_overwrite() {
    let mut stream = ByteStream::new(2);

    assert_eq!(stream.write(b"cat"), 2);
    stream.pop_output(2);
    assert_eq!(stream.write(b"tac"), 2);

    assert_eq!(stream.input_ended(), false);
    assert_eq!(stream.buffer_empty(), false);
    assert_eq!(stream.eof(), false);
    assert_eq!(stream.bytes_read(), 2);
    assert_eq!(stream.bytes_written(), 4);
    assert_eq!(stream.remaining_capacity(), 0);
    assert_eq!(stream.buffer_size(), 2);
    assert_eq!(stream.peek_output(2), b"ta");
}
#[test]
fn overwrite_pop_overwrite() {
    let mut stream = ByteStream::new(2);
    assert_eq!(stream.write(b"cat"), 2);
    stream.pop_output(1);
    assert_eq!(stream.write(b"tac"), 1);

    assert_eq!(stream.input_ended(), false);
    assert_eq!(stream.buffer_empty(), false);
    assert_eq!(stream.eof(), false);
    assert_eq!(stream.bytes_read(), 1);
    assert_eq!(stream.bytes_written(), 3);
    assert_eq!(stream.remaining_capacity(), 0);
    assert_eq!(stream.buffer_size(), 2);
    assert_eq!(stream.peek_output(2), b"at");
}
#[test]
fn long_stream() {
    let mut stream = ByteStream::new(3);

    assert_eq!(stream.write(b"abcdef"), 3);
    assert_eq!(stream.peek_output(3), b"abc");
    stream.pop_output(1);

    for _ in 0..99997 {
        assert_eq!(stream.remaining_capacity(), 1);
        assert_eq!(stream.buffer_size(), 2);
        assert_eq!(stream.write(b"abc"), 1);
        assert_eq!(stream.remaining_capacity(), 0);
        assert_eq!(stream.peek_output(3), b"bca");
        stream.pop_output(1);

        assert_eq!(stream.remaining_capacity(), 1);
        assert_eq!(stream.buffer_size(), 2);
        assert_eq!(stream.write(b"bca"), 1);
        assert_eq!(stream.remaining_capacity(), 0);
        assert_eq!(stream.peek_output(3), b"cab");
        stream.pop_output(1);

        assert_eq!(stream.remaining_capacity(), 1);
        assert_eq!(stream.buffer_size(), 2);
        assert_eq!(stream.write(b"cab"), 1);
        assert_eq!(stream.remaining_capacity(), 0);
        assert_eq!(stream.peek_output(3), b"abc");
        stream.pop_output(1);
    }

    stream.end_input();
    assert_eq!(stream.peek_output(2), b"bc");
    stream.pop_output(2);
    assert_eq!(stream.eof(), true);
}
