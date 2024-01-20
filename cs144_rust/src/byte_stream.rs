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
