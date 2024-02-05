use std::{io::IoSlice, rc::Rc};

// 引用计数的只读字符串
#[derive(Clone)]
pub struct Buffer {
    storage: Rc<String>, // cs144 是单线程
    starting_offset: usize,
}

impl Buffer {
    pub fn new(data: String) -> Self {
        Buffer {
            storage: Rc::new(data),
            starting_offset: 0,
        }
    }

    /**
     * 返回字符串的切片, 如果 data 为空, 则返回一个空字符串的引用,不为空返回从 offset 到结尾
     */
    pub fn as_str(&self) -> &str {
        if self.storage.is_empty() {
            ""
        } else {
            &self.storage[self.starting_offset..]
        }
    }

    /**
     * 取 i 位置的字节(与 offset 无关), 有可能越过边界
     */
    pub fn at(&self, i: usize) -> Option<u8> {
        let bytes = self.storage.as_bytes();
        if self.starting_offset + i < bytes.len() {
            Some(bytes[i])
        } else {
            None
        }
    }

    pub fn size(&self) -> usize {
        self.storage.len()
    }

    pub fn is_empty(&self) -> bool {
        self.size() == 0
    }

    pub fn remove_prefix(&mut self, n: usize) {
        if n > self.size() {
            panic!("Buffer::remove_prefix out of bounds");
        }
        self.starting_offset += n;
        // 如果 storage 为空, 且 starting_offset 等于 size, 则将 storage 设置为一个空字符串
        if self.storage.is_empty() && (self.starting_offset == self.size()) {
            self.storage = Rc::new("".to_string());
        }
    }
}

impl Into<&str> for Buffer {
    fn into(self) -> &'static str {
        self.as_str()
    }
}

// A reference-counted discontiguous string that can discard bytes from the front
#[derive(Default, Clone)]
pub struct BufferList {
    buffers: Vec<Buffer>,
}

impl BufferList {
    fn append(&mut self, other: &Self) {
        for buffer in &other.buffers {
            self.buffers.push(buffer.clone());
        }
    }

    fn len(&self) -> usize {
        self.buffers.iter().map(|b| b.len()).sum()
    }

    fn is_empty(&self) -> bool {
        self.buffers.iter().all(|b| b.is_empty())
    }

    fn remove_prefix(&mut self, mut n: usize) {
        while n > 0 {
            if let Some(first) = self.buffers.first_mut() {
                if n < first.len() {
                    first.remove_prefix(n);
                    break;
                } else {
                    n -= first.len();
                    self.buffers.remove(0);
                }
            } else {
                panic!("BufferList::remove_prefix out of bounds");
            }
        }
    }

    fn concatenate(&self) -> String {
        let mut result = String::with_capacity(self.len());
        for buffer in &self.buffers {
            result.push_str(buffer.as_str());
        }
        result
    }
}

// A non-owning temporary view of a discontiguous string
pub struct BufferViewList<'a> {
    views: Vec<&'a str>,
}

impl<'a> BufferViewList<'a> {
    fn new(buffers: &'a BufferList) -> Self {
        let views = buffers.buffers.iter().map(|b| b.as_str()).collect();
        BufferViewList { views }
    }

    fn len(&self) -> usize {
        self.views.iter().map(|v| v.len()).sum()
    }

    fn is_empty(&self) -> bool {
        self.views.iter().all(|v| v.is_empty())
    }

    fn remove_prefix(&mut self, mut n: usize) {
        while n > 0 {
            if let Some(first) = self.views.first_mut() {
                if n < first.len() {
                    let new_first = &first[n..];
                    *first = new_first;
                    break;
                } else {
                    n -= first.len();
                    self.views.remove(0);
                }
            } else {
                panic!("BufferViewList::remove_prefix out of bounds");
            }
        }
    }

    fn as_io_slices(&self) -> Vec<IoSlice> {
        self.views
            .iter()
            .map(|&v| IoSlice::new(v.as_bytes()))
            .collect()
    }
}
