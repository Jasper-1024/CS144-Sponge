use cs144_rust::byte_stream::{ByteStream, ByteStreamTrait};

// fourth part one write
#[test]
fn write_end_pop() {
    let mut stream = ByteStream::new(15);

    stream.write(b"cat");

    assert_eq!(stream.input_ended(), false);
    assert_eq!(stream.buffer_empty(), false);
    assert_eq!(stream.eof(), false);
    assert_eq!(stream.bytes_read(), 0);
    assert_eq!(stream.bytes_written(), 3);
    assert_eq!(stream.remaining_capacity(), 12);
    assert_eq!(stream.buffer_size(), 3);
    assert_eq!(stream.peek_output(3), b"cat");

    stream.end_input();

    assert_eq!(stream.input_ended(), true);
    assert_eq!(stream.buffer_empty(), false);
    assert_eq!(stream.eof(), false);
    assert_eq!(stream.bytes_read(), 0);
    assert_eq!(stream.bytes_written(), 3);
    assert_eq!(stream.remaining_capacity(), 12);
    assert_eq!(stream.buffer_size(), 3);
    assert_eq!(stream.peek_output(3), b"cat");

    stream.pop_output(3);

    assert_eq!(stream.input_ended(), true);
    assert_eq!(stream.buffer_empty(), true);
    assert_eq!(stream.eof(), true);
    assert_eq!(stream.bytes_read(), 3);
    assert_eq!(stream.bytes_written(), 3);
    assert_eq!(stream.remaining_capacity(), 15);
    assert_eq!(stream.buffer_size(), 0);
}
#[test]
fn write_pop_end() {
    let mut stream = ByteStream::new(15);

    // 写入数据 "cat"
    stream.write(b"cat");

    // 执行断言检查
    assert_eq!(stream.input_ended(), false);
    assert_eq!(stream.buffer_empty(), false);
    assert_eq!(stream.eof(), false);
    assert_eq!(stream.bytes_read(), 0);
    assert_eq!(stream.bytes_written(), 3);
    assert_eq!(stream.remaining_capacity(), 12);
    assert_eq!(stream.buffer_size(), 3);
    assert_eq!(stream.peek_output(3), b"cat");

    // 弹出数据
    stream.pop_output(3);

    // 执行断言检查
    assert_eq!(stream.input_ended(), false);
    assert_eq!(stream.buffer_empty(), true);
    assert_eq!(stream.eof(), false);
    assert_eq!(stream.bytes_read(), 3);
    assert_eq!(stream.bytes_written(), 3);
    assert_eq!(stream.remaining_capacity(), 15);
    assert_eq!(stream.buffer_size(), 0);

    // 结束输入
    stream.end_input();

    // 执行最终的断言检查
    assert_eq!(stream.input_ended(), true);
    assert_eq!(stream.buffer_empty(), true);
    assert_eq!(stream.eof(), true);
    assert_eq!(stream.bytes_read(), 3);
    assert_eq!(stream.bytes_written(), 3);
    assert_eq!(stream.remaining_capacity(), 15);
    assert_eq!(stream.buffer_size(), 0);
}
#[test]
fn write_pop2_end() {
    let mut stream = ByteStream::new(15);

    // 写入数据 "cat"
    stream.write(b"cat");

    // 执行断言检查
    assert_eq!(stream.input_ended(), false);
    assert_eq!(stream.buffer_empty(), false);
    assert_eq!(stream.eof(), false);
    assert_eq!(stream.bytes_read(), 0);
    assert_eq!(stream.bytes_written(), 3);
    assert_eq!(stream.remaining_capacity(), 12);
    assert_eq!(stream.buffer_size(), 3);
    assert_eq!(stream.peek_output(3), b"cat");

    // 弹出1个字节
    stream.pop_output(1);

    // 执行断言检查
    assert_eq!(stream.input_ended(), false);
    assert_eq!(stream.buffer_empty(), false);
    assert_eq!(stream.eof(), false);
    assert_eq!(stream.bytes_read(), 1);
    assert_eq!(stream.bytes_written(), 3);
    assert_eq!(stream.remaining_capacity(), 13);
    assert_eq!(stream.buffer_size(), 2);
    assert_eq!(stream.peek_output(2), b"at");

    // 弹出剩余2个字节
    stream.pop_output(2);

    // 执行断言检查
    assert_eq!(stream.input_ended(), false);
    assert_eq!(stream.buffer_empty(), true);
    assert_eq!(stream.eof(), false);
    assert_eq!(stream.bytes_read(), 3);
    assert_eq!(stream.bytes_written(), 3);
    assert_eq!(stream.remaining_capacity(), 15);
    assert_eq!(stream.buffer_size(), 0);

    // 结束输入
    stream.end_input();

    // 执行最终的断言检查
    assert_eq!(stream.input_ended(), true);
    assert_eq!(stream.buffer_empty(), true);
    assert_eq!(stream.eof(), true);
    assert_eq!(stream.bytes_read(), 3);
    assert_eq!(stream.bytes_written(), 3);
    assert_eq!(stream.remaining_capacity(), 15);
    assert_eq!(stream.buffer_size(), 0);
}
