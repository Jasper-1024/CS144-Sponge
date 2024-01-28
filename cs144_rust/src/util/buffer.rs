use std::io::IoSlice;
use std::sync::Arc;

// 引用计数的只读字符串
#[derive(Clone)]
pub struct Buffer {
    storage: Arc<String>, // 跨线程共享 String
    starting_offset: usize,
}

impl Buffer {
    fn new(data: String) -> Self {
        Buffer {
            storage: Arc::new(data),
            starting_offset: 0,
        }
    }

    fn len(&self) -> usize {
        self.storage.len() - self.starting_offset
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn as_str(&self) -> &str {
        &self.storage[self.starting_offset..]
    }

    fn remove_prefix(&mut self, n: usize) {
        if n > self.len() {
            panic!("Buffer::remove_prefix out of bounds");
        }
        self.starting_offset += n;
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
