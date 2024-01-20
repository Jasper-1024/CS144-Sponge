use cs144_rust::byte_stream::{ByteStream, ByteStreamTrait};
#[test]
fn write_write_end_pop_pop() {
    let mut stream = ByteStream::new(15);

    // 第一次写入数据 "cat"
    stream.write(b"cat");

    // 断言检查
    assert_eq!(stream.input_ended(), false);
    assert_eq!(stream.buffer_empty(), false);
    assert_eq!(stream.eof(), false);
    assert_eq!(stream.bytes_read(), 0);
    assert_eq!(stream.bytes_written(), 3);
    assert_eq!(stream.remaining_capacity(), 12);
    assert_eq!(stream.buffer_size(), 3);
    assert_eq!(stream.peek_output(3), b"cat");

    // 第二次写入数据 "tac"
    stream.write(b"tac");

    // 断言检查
    assert_eq!(stream.input_ended(), false);
    assert_eq!(stream.buffer_empty(), false);
    assert_eq!(stream.eof(), false);
    assert_eq!(stream.bytes_read(), 0);
    assert_eq!(stream.bytes_written(), 6);
    assert_eq!(stream.remaining_capacity(), 9);
    assert_eq!(stream.buffer_size(), 6);
    assert_eq!(stream.peek_output(6), b"cattac");

    // 结束输入
    stream.end_input();

    // 断言检查
    assert_eq!(stream.input_ended(), true);
    assert_eq!(stream.buffer_empty(), false);
    assert_eq!(stream.eof(), false);
    assert_eq!(stream.bytes_read(), 0);
    assert_eq!(stream.bytes_written(), 6);
    assert_eq!(stream.remaining_capacity(), 9);
    assert_eq!(stream.buffer_size(), 6);
    assert_eq!(stream.peek_output(6), b"cattac");

    // 弹出前2个字节
    stream.pop_output(2);

    // 断言检查
    assert_eq!(stream.input_ended(), true);
    assert_eq!(stream.buffer_empty(), false);
    assert_eq!(stream.eof(), false);
    assert_eq!(stream.bytes_read(), 2);
    assert_eq!(stream.bytes_written(), 6);
    assert_eq!(stream.remaining_capacity(), 11);
    assert_eq!(stream.buffer_size(), 4);
    assert_eq!(stream.peek_output(4), b"ttac");

    // 弹出剩余4个字节
    stream.pop_output(4);

    // 断言检查
    assert_eq!(stream.input_ended(), true);
    assert_eq!(stream.buffer_empty(), true);
    assert_eq!(stream.eof(), true);
    assert_eq!(stream.bytes_read(), 6);
    assert_eq!(stream.bytes_written(), 6);
    assert_eq!(stream.remaining_capacity(), 15);
    assert_eq!(stream.buffer_size(), 0);
}

#[test]
fn write_pop_write_end_pop() {
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

    // 弹出2个字节
    stream.pop_output(2);

    // 执行断言检查
    assert_eq!(stream.input_ended(), false);
    assert_eq!(stream.buffer_empty(), false);
    assert_eq!(stream.eof(), false);
    assert_eq!(stream.bytes_read(), 2);
    assert_eq!(stream.bytes_written(), 3);
    assert_eq!(stream.remaining_capacity(), 14);
    assert_eq!(stream.buffer_size(), 1);
    assert_eq!(stream.peek_output(1), b"t");

    // 写入数据 "tac"
    stream.write(b"tac");

    // 执行断言检查
    assert_eq!(stream.input_ended(), false);
    assert_eq!(stream.buffer_empty(), false);
    assert_eq!(stream.eof(), false);
    assert_eq!(stream.bytes_read(), 2);
    assert_eq!(stream.bytes_written(), 6);
    assert_eq!(stream.remaining_capacity(), 11);
    assert_eq!(stream.buffer_size(), 4);
    assert_eq!(stream.peek_output(4), b"ttac");

    // 结束输入
    stream.end_input();

    // 执行断言检查
    assert_eq!(stream.input_ended(), true);
    assert_eq!(stream.buffer_empty(), false);
    assert_eq!(stream.eof(), false);
    assert_eq!(stream.bytes_read(), 2);
    assert_eq!(stream.bytes_written(), 6);
    assert_eq!(stream.remaining_capacity(), 11);
    assert_eq!(stream.buffer_size(), 4);
    assert_eq!(stream.peek_output(4), b"ttac");

    // 弹出所有4个字节
    stream.pop_output(4);

    // 执行最终的断言检查
    assert_eq!(stream.input_ended(), true);
    assert_eq!(stream.buffer_empty(), true);
    assert_eq!(stream.eof(), true);
    assert_eq!(stream.bytes_read(), 6);
    assert_eq!(stream.bytes_written(), 6);
    assert_eq!(stream.remaining_capacity(), 15);
    assert_eq!(stream.buffer_size(), 0);
}
