use std::collections::BTreeMap;

use crate::byte_stream::{ByteStream, ByteStreamTrait};

pub struct StreamReassembler<'a> {
    pub(crate) capacity: usize, // The maximum number of bytes

    unassembled: BTreeMap<u64, (&'a [u8], u64)>,
    bytes_stream: ByteStream,
    // all index
    assembled: u64,
    unassembled_bytes: usize,
    eof_index: u64,
}

pub trait StreamReassemblerTrait<'a> {
    fn new(capacity: usize) -> Self; // 创建最大容量为 capacity 字节
    fn push_substring(&'a mut self, data: &'a [u8], index: u64, eof: bool); // 流重组
    fn stream_out(&self) -> &ByteStream; //输出已经重组的流
    fn unassembled_bytes(&self) -> usize; // 返回还未重组的字节数
    fn is_empty(&self) -> bool; //是否有未重组的字节
}

impl<'a> StreamReassemblerTrait<'a> for StreamReassembler<'a> {
    fn new(capacity: usize) -> Self {
        StreamReassembler {
            capacity,
            unassembled: BTreeMap::new(),
            bytes_stream: ByteStream::new(capacity),
            assembled: 0,
            unassembled_bytes: 0,
            eof_index: 0,
        }
    }
    fn push_substring(&'a mut self, mut data: &'a [u8], index: u64, eof: bool) {
        let mut base = 0 as i64;
        if index > (self.assembled + self.capacity as u64) {
            return; // 超出容量
        }
        if index + data.len() as u64 > (self.assembled + self.capacity as u64) {
            // 截断 超出容量部分
            data = &data[..(self.assembled + self.capacity as u64 - index) as usize];
        }

        // 如果 index 重复
        if let Some((temp_data, _)) = self.unassembled.get(&index) {
            // 是另一个报文的碎片 | 重复的报文
            if temp_data.len() <= data.len() {
                return;
            } else {
                base -= temp_data.len() as i64;
                // 之前存了碎片,移除,继续存储
                self.unassembled.remove(&index);
            }
        }
        let mut index = index;
        // 范围查找 unassembled 的 [assembled, index) 的所有数据
        for (_, (_, next_index)) in self.unassembled.range_mut(self.assembled..index) {
            // 未重叠
            if *next_index <= index {
                continue;
            }
            // data 只是个 碎片 | 如果相等,则只差一个字节
            if *next_index > (index + data.len() as u64) {
                return;
            }
            index = *next_index; // 更新 index
            data = &data[(*next_index - index - 1) as usize..]; // 更新 data | 索引从 0 开始
        }

        let next_key = index + data.len() as u64 + 1;
        let mut keys_to_remove = Vec::new();
        // 范围查找 unassembled 的 [index, index + data.len()+1) 的所有数据, key 已经是在其中了
        for (key, (_, next_index)) in self
            .unassembled
            .range_mut(index..(index + data.len() as u64 + 1))
        {
            // key 被完全覆盖
            if *next_index <= next_key {
                // self.unassembled.remove(&key);
                keys_to_remove.push(*key); // 被完全覆盖,待移除
                continue;
            }
            data = &data[..(*key - index - 1) as usize]; // 更新 data | 索引从 0 开始 | 截后半段
        }
        for key in keys_to_remove {
            self.unassembled.remove(&key);
            base -= data.len() as i64;
        }

        if data.len() == 0 {
            return; // data 为空
        };
        self.unassembled
            .insert(index, (data, index + data.len() as u64 + 1)); // 插入
        base += data.len() as i64;

        // 更多有序的数据
        while let Some((data, next_index)) = self.unassembled.get(&self.assembled) {
            self.bytes_stream.write(data);
            self.assembled = *next_index;
            base -= data.len() as i64;
        }

        self.unassembled_bytes += base as usize; // 更新 unassembled_bytes
        if eof {
            self.eof_index = index + data.len() as u64;
        }
        if self.eof_index <= self.assembled {
            self.bytes_stream.end_input();
        }
    }

    fn stream_out(&self) -> &ByteStream {
        return &self.bytes_stream;
    }

    fn unassembled_bytes(&self) -> usize {
        return self.unassembled_bytes;
    }
    fn is_empty(&self) -> bool {
        return self.unassembled_bytes == 0;
    }
}
