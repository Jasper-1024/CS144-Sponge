use std::{collections::VecDeque, io::IoSlice, rc::Rc};

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

    pub fn len(&self) -> usize {
        self.as_str().len()
    }

    pub fn remove_prefix(&mut self, n: usize) -> Result<(), &'static str> {
        if n > self.len() {
            return Err("Buffer::remove_prefix out of bounds");
        }
        self.starting_offset += n;
        // 如果 storage 为空, 且 starting_offset 等于 size, 则将 storage 设置为一个空字符串
        if self.storage.is_empty() && (self.starting_offset == self.len()) {
            self.storage = Rc::new("".to_string());
        }
        Ok(())
    }
}

// as_ref 方法返回一个字符串的引用
impl AsRef<str> for Buffer {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

// A reference-counted discontiguous string that can discard bytes from the front
#[derive(Default, Clone)]
pub struct BufferList {
    buffers: VecDeque<Buffer>,
}

impl BufferList {
    /**
     * 这里会有所有权的转移.
     */
    fn append(&mut self, other: &mut Self) {
        self.buffers.append(&mut other.buffers);
    }

    // size... rust used len more often..
    fn len(&self) -> usize {
        self.buffers.iter().map(|b| b.len()).sum()
    }

    fn buffers(&self) -> &VecDeque<Buffer> {
        &self.buffers
    }

    /**
     * to buffer
     */
    fn to_buffer(&self) -> Result<Option<Buffer>, &'static str> {
        match self.buffers.len() {
            0 => Ok(None),
            1 => Ok(Some(self.buffers[0].clone())),
            _ => Err("BufferList: please use concatenate() to combine a multi-Buffer BufferList into one Buffer"),
        }
    }
    /**
     * remove prefix from the front of the buffer list
     */
    fn remove_prefix(&mut self, mut n: usize) -> Result<(), &'static str> {
        while n > 0 {
            // first element
            if let Some(first) = self.buffers.front_mut() {
                if n < first.len() {
                    let _ = first.remove_prefix(n);
                    break;
                } else {
                    n -= first.len();
                    self.buffers.remove(0);
                }
            } else {
                // first element is None
                return Err("BufferList::remove_prefix out of bounds");
            }
        }
        Ok(())
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
    views: VecDeque<&'a str>,
}

impl<'a> BufferViewList<'a> {
    pub fn new(buffers: &'a BufferList) -> Self {
        let views = buffers.buffers.iter().map(|b| b.as_str()).collect();
        BufferViewList { views }
    }

    pub fn new_str(s: &'a str) -> Self {
        let mut views = VecDeque::new();
        views.push_back(s);
        BufferViewList { views }
    }
    /**
     * 所有权限制,只能传入 String 的引用
     */
    pub fn new_string(s: &'a String) -> Self {
        let mut views = VecDeque::new();
        views.push_back(s.as_str());
        BufferViewList { views }
    }

    pub fn len(&self) -> usize {
        self.views.iter().map(|v| v.len()).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.views.iter().all(|v| v.is_empty())
    }

    pub fn remove_prefix(&mut self, mut n: usize) -> Result<(), &'static str> {
        while n > 0 {
            if let Some(first) = self.views.front_mut() {
                if n < first.len() {
                    let new_first = &first[n..];
                    *first = new_first;
                    break;
                } else {
                    n -= first.len();
                    self.views.remove(0);
                }
            } else {
                return Err("BufferViewList::remove_prefix out of bounds");
            }
        }
        Ok(())
    }

    pub fn as_io_slices(&self) -> Vec<IoSlice> {
        self.views
            .iter()
            .map(|&v| IoSlice::new(v.as_bytes()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let buffer = Buffer::new("Hello, world!".to_string());
        assert_eq!(buffer.as_str(), "Hello, world!");
    }

    #[test]
    fn test_as_str() {
        let buffer = Buffer::new("Hello, world!".to_string());
        assert_eq!(buffer.as_str(), "Hello, world!");
    }

    #[test]
    fn test_at() {
        let buffer = Buffer::new("Hello, world!".to_string());
        assert_eq!(buffer.at(0), Some(b'H'));
        assert_eq!(buffer.at(12), Some(b'!'));
        assert_eq!(buffer.at(13), None);
    }

    #[test]
    fn test_size() {
        let buffer = Buffer::new("Hello, world!".to_string());
        assert_eq!(buffer.len(), 13);
    }

    #[test]
    fn test_remove_prefix() {
        let mut buffer = Buffer::new("Hello, world!".to_string());
        assert_eq!(buffer.remove_prefix(6), Ok(()));
        assert_eq!(buffer.as_str(), " world!");
        assert_eq!(buffer.remove_prefix(7), Ok(()));
        assert_eq!(buffer.as_str(), "");
        assert_eq!(
            buffer.remove_prefix(1),
            Err("Buffer::remove_prefix out of bounds")
        );
    }

    /// BufferList tests

    #[test]
    fn test_append() {
        let mut buffer_list1 = BufferList {
            buffers: VecDeque::new(),
        };
        let mut buffer_list2 = BufferList {
            buffers: VecDeque::new(),
        };

        buffer_list1
            .buffers
            .push_back(Buffer::new("Hello".to_string()));
        buffer_list2
            .buffers
            .push_back(Buffer::new(" World".to_string()));

        buffer_list1.append(&mut buffer_list2);

        assert_eq!(buffer_list1.buffers.len(), 2);
        assert_eq!(buffer_list1.concatenate(), "Hello World");
    }

    #[test]
    fn test_len() {
        let mut buffer_list = BufferList {
            buffers: VecDeque::new(),
        };
        buffer_list
            .buffers
            .push_back(Buffer::new("Hello".to_string()));
        buffer_list
            .buffers
            .push_back(Buffer::new(" World".to_string()));

        assert_eq!(buffer_list.len(), 11);
    }

    #[test]
    fn test_to_buffer() {
        let mut buffer_list = BufferList {
            buffers: VecDeque::new(),
        };
        buffer_list
            .buffers
            .push_back(Buffer::new("Hello".to_string()));

        let buffer = buffer_list.to_buffer().unwrap().unwrap();
        assert_eq!(buffer.as_str(), "Hello");
    }

    #[test]
    fn test_remove_prefix_buffer_list() {
        let mut buffer_list = BufferList {
            buffers: VecDeque::new(),
        };
        buffer_list
            .buffers
            .push_back(Buffer::new("Hello".to_string()));
        buffer_list
            .buffers
            .push_back(Buffer::new(" World".to_string()));

        buffer_list.remove_prefix(6).unwrap();
        assert_eq!(buffer_list.concatenate(), "World");
    }

    #[test]
    fn test_concatenate() {
        let mut buffer_list = BufferList {
            buffers: VecDeque::new(),
        };
        buffer_list
            .buffers
            .push_back(Buffer::new("Hello".to_string()));
        buffer_list
            .buffers
            .push_back(Buffer::new(" World".to_string()));

        assert_eq!(buffer_list.concatenate(), "Hello World");
    }

    /// BufferViewList tests
    #[test]
    fn test_new_bufferviewlist() {
        let mut buffer_list = BufferList::default();
        buffer_list
            .buffers
            .push_back(Buffer::new("Hello".to_string()));
        buffer_list
            .buffers
            .push_back(Buffer::new(", world!".to_string()));

        let view_list = BufferViewList::new(&buffer_list);

        assert_eq!(view_list.len(), 13);
        assert!(!view_list.is_empty());
    }

    #[test]
    fn test_new_str() {
        let view_list = BufferViewList::new_str("Hello, world!");

        assert_eq!(view_list.len(), 13);
        assert!(!view_list.is_empty());
    }

    #[test]
    fn test_new_string() {
        let s = "Hello, world!".to_string();
        let view_list = BufferViewList::new_string(&s);

        assert_eq!(view_list.len(), 13);
        assert!(!view_list.is_empty());
    }

    #[test]
    fn test_remove_prefix_bufferviewlist() {
        let mut view_list = BufferViewList::new_str("Hello, world!");

        assert_eq!(view_list.len(), 13);
        assert!(!view_list.is_empty());

        view_list.remove_prefix(6).unwrap();

        assert_eq!(view_list.len(), 7);
        assert_eq!(*view_list.views.front().unwrap(), " world!");
    }

    #[test]
    fn test_remove_prefix_out_of_bounds() {
        let mut view_list = BufferViewList::new_str("Hello, world!");

        assert_eq!(view_list.len(), 13);
        assert!(!view_list.is_empty());

        let result = view_list.remove_prefix(20);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "BufferViewList::remove_prefix out of bounds"
        );
    }
}
