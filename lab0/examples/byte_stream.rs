pub struct ByteStream {
    // Your code here -- add private members as necessary.
    pub error: bool, // Flag indicating that the stream suffered an error.
    // private members
    buffer: Vec<u8>,    // Buffer for storing bytes.
    capacity: usize,    // Capacity of the buffer.
    read_index: usize,  // Index of the next byte to read.
    write_index: usize, // Index of the next byte to write.
    end: bool,
    err: bool,
    read_count: usize,  // Number of bytes read.
    write_count: usize, // Number of bytes written.
}

pub trait ByteStreamTrait {
    // 创建一个新的字节流，容量为 `capacity`
    fn new(capacity: usize) -> Self;

    // 写入字节到流中，返回实际写入的字节数
    fn write(&mut self, data: &[u8]) -> usize;

    // 返回流还能接受的字节数
    fn remaining_capacity(&self) -> usize;

    // 结束输入
    fn end_input(&mut self);

    // 设置错误标志
    fn set_error(&mut self);

    // 输出流中 len 个字节的副本
    fn peek_output(&self, len: usize) -> Vec<u8>;

    // 从流中移除 len 个字节
    fn pop_output(&mut self, len: usize);

    // 读取（复制并移除）流中的 len 个字节
    fn read(&mut self, len: usize) -> Vec<u8>;

    // 返回输入是否已结束
    fn input_ended(&self) -> bool;

    // 返回是否发生了错误
    fn error(&self) -> bool;

    // 返回当前可以从流中读取的最大字节数
    fn buffer_size(&self) -> usize;

    // 返回缓冲区是否为空
    fn buffer_empty(&self) -> bool;

    // 返回输出是否已结束
    fn eof(&self) -> bool;

    // 返回已写入的总字节数
    fn bytes_written(&self) -> usize;

    // 返回已读取的总字节数
    fn bytes_read(&self) -> usize;
}

impl ByteStream {
    fn real_capacity(&self) -> usize {
        self.capacity + 1
    }
}

impl ByteStreamTrait for ByteStream {
    // your code here
    fn new(capacity: usize) -> Self {
        ByteStream {
            error: false,
            buffer: vec![0; capacity + 1],
            capacity: capacity,
            read_index: 0,
            write_index: 0,
            end: false,
            err: false,
            read_count: 0,
            write_count: 0,
        }
    }
    // 环形缓冲区 一次性写入全部可写空间
    fn write(&mut self, data: &[u8]) -> usize {
        if self.end || self.err {
            // 输入结束或者发生错误
            return 0;
        }
        let len = data.len().min(self.remaining_capacity()); // 写入的总字节数
        let end_len = self.real_capacity() - self.write_index; // 若 data 需要分割的位置

        let (first, second) = data.split_at(end_len.min(len)); // 分割为两部分
        self.buffer[self.write_index..self.write_index + first.len()].copy_from_slice(first); // 写入第一部分
        self.write_index = (self.write_index + first.len()) % self.real_capacity(); // 更新写入位置

        if len > first.len() && !second.is_empty() {
            // 写入第二部分
            self.buffer[0..(len - first.len())].copy_from_slice(&second[0..(len - first.len())]);
            self.write_index = len - first.len(); // 更新写入位置
        }
        self.write_count += len; // 更新已写入的字节数
        len
    }

    fn remaining_capacity(&self) -> usize {
        // if self.end || self.err {
        //     // 输入结束或者发生错误
        //     return 0;
        // }

        let buffer_size: usize = self.real_capacity();
        if self.write_index == self.read_index {
            // The buffer is empty
            return buffer_size - 1; // capacity
        }
        if (self.write_index + 1) % buffer_size == self.read_index {
            // The buffer is full
            return 0;
        }

        if self.write_index > self.read_index {
            buffer_size - (self.write_index - self.read_index) - 1
        } else {
            self.read_index - self.write_index - 1
        }
    }

    fn end_input(&mut self) {
        self.end = true;
    }

    fn set_error(&mut self) {
        self.err = true;
    }

    fn peek_output(&self, len: usize) -> Vec<u8> {
        if self.err {
            return vec![];
        }
        let len = len.min(self.buffer_size()); // 可读取的最大字节数
        let end_len = self.real_capacity() - self.read_index; // 从读取位置到缓冲区末尾的字节数
        let mut res = vec![0; len];
        let (first, second) = res.split_at_mut(end_len.min(len)); // 分割为两部分
        first.copy_from_slice(&self.buffer[self.read_index..self.read_index + first.len()]); // 读取第一部分
        if !second.is_empty() {
            second.copy_from_slice(&self.buffer[0..second.len()]); // 读取第二部分
        }
        res
    }
    // 环形缓冲区 pop 只是移动读取位置
    fn pop_output(&mut self, len: usize) {
        if self.err {
            return;
        }
        let len = len.min(self.buffer_size()); // 可读取的最大字节数
        self.read_index = (self.read_index + len) % self.real_capacity(); // 更新读取位置
        self.read_count += len; // 更新已读取的字节数
    }
    // 调用 已有实现
    fn read(&mut self, len: usize) -> Vec<u8> {
        let res = self.peek_output(len);
        self.pop_output(len);
        res
    }
    fn input_ended(&self) -> bool {
        self.end
    }
    fn error(&self) -> bool {
        self.err
    }

    fn buffer_size(&self) -> usize {
        if self.err {
            return 0;
        }
        if self.write_index == self.read_index {
            // 缓冲区为空
            return 0;
        }
        // 写入位置在读取位置之后
        if self.write_index > self.read_index {
            self.write_index - self.read_index
        } else {
            // 写入位置在读取位置之前
            self.real_capacity() - self.read_index + self.write_index
        }
    }

    fn buffer_empty(&self) -> bool {
        self.buffer_size() == 0
    }

    fn eof(&self) -> bool {
        self.end && self.buffer_empty()
    }
    // 已写入的字节数
    fn bytes_written(&self) -> usize {
        self.write_count
    }
    // 已读取的字节数
    fn bytes_read(&self) -> usize {
        self.read_count
    }
}

fn main() {}

#[cfg(test)]
mod tests {
    use super::*;

    // first part stream capacity
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

    // second part stream construction
    #[test]
    fn construction() {
        let stream = ByteStream::new(15);

        assert_eq!(stream.input_ended(), false);
        assert_eq!(stream.buffer_empty(), true);
        assert_eq!(stream.eof(), false);
        assert_eq!(stream.bytes_read(), 0);
        assert_eq!(stream.bytes_written(), 0);
        assert_eq!(stream.remaining_capacity(), 15);
        assert_eq!(stream.buffer_size(), 0);
    }

    #[test]
    fn construction_end() {
        let mut stream = ByteStream::new(15);

        stream.end_input();
        assert_eq!(stream.input_ended(), true);
        assert_eq!(stream.buffer_empty(), true);
        assert_eq!(stream.eof(), true);
        assert_eq!(stream.bytes_read(), 0);
        assert_eq!(stream.bytes_written(), 0);
        assert_eq!(stream.remaining_capacity(), 15);
        assert_eq!(stream.buffer_size(), 0);
    }
    // third part many writes
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
            assert_eq!(stream.bytes_written(), acc);
            assert_eq!(stream.remaining_capacity(), CAPACITY - acc);
            assert_eq!(stream.buffer_size(), acc);
        }
    }

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

    // fifth part two writes
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
}
