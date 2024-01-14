use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use crate::byte_stream::{self, ByteStream, ByteStreamTrait};

pub struct StreamReassembler<'a> {
    pub(crate) capacity: usize, // The maximum number of bytes

    unassembled: BTreeMap<u64, (&'a [u8], u64)>,
    bytes_stream: Rc<RefCell<ByteStream>>,
    // all index
    unassembled_bytes: usize, // 此刻未重组的字节数
    assembled: u64,           // 下一个要重组的索引开始位置
    eof_index: u64,           // 最终字节流 eof 对应索引
}

pub trait StreamReassemblerTrait<'a> {
    fn new(capacity: usize) -> Self; // 创建最大容量为 capacity 字节
    fn push_substring(&mut self, data: &'a [u8], index: u64, eof: bool); // 流重组
    fn stream_out(&self) -> Rc<RefCell<ByteStream>>; // 返回重组后的流
    fn unassembled_bytes(&self) -> usize; // 返回还未重组的字节数
    fn is_empty(&self) -> bool; //是否有未重组的字节
}

impl<'a> StreamReassemblerTrait<'a> for StreamReassembler<'a> {
    fn new(capacity: usize) -> Self {
        StreamReassembler {
            capacity,
            unassembled: BTreeMap::new(),
            bytes_stream: Rc::new(ByteStream::new(capacity).into()),
            assembled: 0,
            unassembled_bytes: 0,
            eof_index: std::u64::MAX,
        }
    }
    fn push_substring(&mut self, mut data: &'a [u8], mut index: u64, eof: bool) {
        if index >= (self.assembled + self.capacity as u64) {
            return; // data.start 在 capacity 之后
        }
        if (index + data.len() as u64) <= self.assembled {
            return; // end < assembled, data 就被重组了
        }

        if index < self.assembled {
            // data.start 在 assembled 已重组的数据之前,截断
            data = &data[(self.assembled - index) as usize..]; // 更新 data | 索引从 0 开始
            index = self.assembled; // 更新 index
        }
        if index + data.len() as u64 > (self.assembled + self.capacity as u64) {
            // 截断 超出容量部分
            data = &data[..(self.assembled + self.capacity as u64 - index) as usize];
        }

        let mut base = 0 as i64;
        let mut keys_to_remove = Vec::new();

        // 如果 index 重复
        if let Some((temp_data, _)) = self.unassembled.get(&index) {
            // 是另一个报文的碎片 | 重复的报文
            if temp_data.len() >= data.len() {
                return;
            } else {
                base -= temp_data.len() as i64; // 更新未重组字节数 base
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

        let mut next_key = index + data.len() as u64;
        // 范围查找 unassembled 的 [index, next_key) 的所有数据, key 已经是在其中了
        for (key, (temp_data, next_index)) in self.unassembled.range_mut(index..next_key) {
            // key 被完全覆盖
            if *next_index <= next_key {
                keys_to_remove.push(*key); // 被完全覆盖,待移除
                base -= temp_data.len() as i64; // 更新未重组字节数 base
                continue;
            }
            data = &data[..(*key - index) as usize]; // 更新 data | 索引从 0 开始 | 截后半段
            next_key = index + data.len() as u64;
        }
        for &key in keys_to_remove.iter() {
            self.unassembled.remove(&key);
        }
        keys_to_remove.clear(); // clear

        if data.len() == 0 {
            return; // data 为空
        };
        self.unassembled
            .insert(index, (data, index + data.len() as u64)); // 插入
        base += data.len() as i64; // 更新未重组字节数 base

        // 更多有序的数据
        while let Some((temp_data, next_index)) = self.unassembled.get(&self.assembled) {
            keys_to_remove.push(self.assembled); // 待删除

            let mut byte_stream = self.bytes_stream.borrow_mut(); // borrow_mut

            if byte_stream.remaining_capacity() == 0 {
                break; // stream 已满
            }

            let write_len = byte_stream.write(temp_data); // 写入成功的字节数
            if write_len == 0 {
                break; // stream 已满 | 无法写入
            }
            keys_to_remove.push(self.assembled); // 待删除

            self.assembled = self.assembled + write_len as u64; // 更新 assembled
            base -= write_len as i64; // 更新未重组字节数 base

            if write_len < temp_data.len() {
                // 只写入了部分
                self.unassembled
                    .insert(self.assembled, (&temp_data[write_len..], *next_index));
                break;
            }
        }
        // 移除已重组的数据
        for &key in keys_to_remove.iter() {
            self.unassembled.remove(&key);
        }

        // 更新 unassembled_bytes
        if base < 0 {
            self.unassembled_bytes -= base.abs() as usize; // rust 这个类型系统..太严格了..😒
        } else {
            self.unassembled_bytes += base as usize;
        }

        if eof {
            self.eof_index = index + data.len() as u64;
        }
        if self.eof_index <= self.assembled {
            self.bytes_stream.borrow_mut().end_input();
        }
    }

    fn stream_out(&self) -> Rc<RefCell<ByteStream>> {
        return self.bytes_stream.clone();
    }

    fn unassembled_bytes(&self) -> usize {
        return self.unassembled_bytes;
    }
    fn is_empty(&self) -> bool {
        return self.unassembled_bytes == 0;
    }
}
