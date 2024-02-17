use std::{collections::VecDeque, io::IoSlice, rc::Rc};

// 引用计数的 只读 u8 数组
#[derive(Clone, Debug, PartialEq)]
pub struct Buffer {
    storage: Rc<[u8]>, // cs144 是单线程
    starting_offset: usize,
}

impl Buffer {
    pub fn new<const N: usize>(data: [u8; N]) -> Self {
        Buffer {
            storage: Rc::from(data),
            starting_offset: 0,
        }
    }

    pub fn new_form_vec(data: Vec<u8>) -> Self {
        Buffer {
            storage: Rc::from(data.into_boxed_slice()),
            starting_offset: 0,
        }
    }

    /**
     * 返回u8切片, 有可能为空
     */
    pub fn as_slice(&self) -> Option<&[u8]> {
        if self.storage.is_empty() {
            None
        } else {
            Some(&self.storage[self.starting_offset..])
        }
    }

    /**
     * 取 i 位置的字节(与 offset 无关), 有可能越过边界
     */
    pub fn at(&self, i: usize) -> Option<u8> {
        let bytes = self.storage.as_ref();
        if self.starting_offset + i < bytes.len() {
            Some(bytes[i])
        } else {
            None
        }
    }

    /**
     * 返回 u8 数组 len, 无效返回 0
     */
    pub fn len(&self) -> usize {
        self.as_slice().map_or(0, |slice| slice.len())
    }

    /**
     * remove prefix from the front of the buffer
     */
    pub fn remove_prefix(&mut self, n: usize) -> Result<(), &'static str> {
        if n > self.len() {
            return Err("Buffer::remove_prefix n too large");
        }
        self.starting_offset += n;
        // 如果 storage 为空, 且 starting_offset 等于 size, 则将 storage 设置为一个空字符串
        if self.storage.is_empty() && (self.starting_offset >= self.len()) {
            self.storage = Rc::from(Vec::new());
            self.starting_offset = 0;
            return Err("Buffer::remove_prefix out of bounds, reset storage to empty string");
        }
        Ok(())
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Buffer {
            storage: Rc::from(Vec::new().into_boxed_slice()),
            starting_offset: 0,
        }
    }
}

// as_ref 方法返回一个 u8 数组的引用
impl AsRef<[u8]> for Buffer {
    fn as_ref(&self) -> &[u8] {
        self.as_slice().unwrap_or_default()
    }
}

impl From<Vec<u8>> for Buffer {
    fn from(data: Vec<u8>) -> Self {
        Buffer {
            storage: Rc::from(data.into_boxed_slice()),
            starting_offset: 0,
        }
    }
}

impl<const N: usize> From<[u8; N]> for Buffer {
    fn from(data: [u8; N]) -> Self {
        Buffer::new(data)
    }
}

// A reference-counted discontiguous string that can discard bytes from the front
#[derive(Default, Clone)]
pub struct BufferList {
    buffers: VecDeque<Buffer>,
}

impl BufferList {
    pub fn new() -> Self {
        BufferList {
            buffers: VecDeque::new(),
        }
    }
    /**
     * 这里会有所有权的转移.
     */
    pub fn append(&mut self, other: &mut Self) {
        self.buffers.append(&mut other.buffers);
    }

    pub fn append_vec(&mut self, data: Vec<u8>) {
        self.buffers.push_back(Buffer {
            storage: Rc::from(data),
            starting_offset: 0,
        });
    }

    pub fn append_buffer(&mut self, buffer: Buffer) {
        self.buffers.push_back(buffer);
    }

    // size... rust used len more often..
    pub fn len(&self) -> usize {
        self.buffers.iter().map(|b| b.len()).sum()
    }

    pub fn buffers(&self) -> &VecDeque<Buffer> {
        &self.buffers
    }

    /**
     * to buffer
     */
    pub fn to_buffer(&self) -> Result<Option<Buffer>, &'static str> {
        match self.buffers.len() {
            0 => Ok(None),
            1 => Ok(Some(self.buffers[0].clone())),
            _ => Err("BufferList: please use concatenate() to combine a multi-Buffer BufferList into one Buffer"),
        }
    }
    /**
     * remove prefix from the front of the buffer list
     */
    pub fn remove_prefix(&mut self, mut n: usize) -> Result<(), &'static str> {
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

    /**
     * make a copy of all buffers
     */
    pub fn concatenate(&self) -> Vec<u8> {
        let mut result = Vec::new();
        for buffer in &self.buffers {
            result.extend_from_slice(buffer.as_ref());
        }
        result
    }
}

// A non-owning temporary view of a discontiguous string
pub struct BufferViewList<'a> {
    views: VecDeque<&'a [u8]>,
}

impl<'a> BufferViewList<'a> {
    /**
     * new BufferViewList from BufferList
     */
    pub fn new(buffers: &'a BufferList) -> Self {
        let views = buffers
            .buffers
            .iter()
            .map(|b| b.as_slice().unwrap_or_default())
            .collect();
        BufferViewList { views }
    }

    /**
     * new BufferViewList from &[u8]
     */
    pub fn new_frome_slice(s: &'a [u8]) -> Self {
        let mut views = VecDeque::new();
        views.push_back(s);
        BufferViewList { views }
    }
    /**
     * new BufferViewList from [u8; N] , not work for lifetime
     */
    // pub fn new_form_list<const N: usize>(s: [u8; N]) -> Self {
    //     let mut views = VecDeque::new();
    //     views.push_back(&s as &[u8]);
    //     BufferViewList { views }
    // }

    pub fn len(&self) -> usize {
        self.views.iter().map(|v| v.len()).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.views.iter().all(|v| v.is_empty())
    }

    /**
     * remove prefix from the front of the buffer list
     */
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
        self.views.iter().map(|&v| IoSlice::new(v)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let buffer = Buffer::new([1, 2, 3, 4, 5]);
        assert_eq!(buffer.storage.len(), 5);
        assert_eq!(buffer.starting_offset, 0);
    }

    #[test]
    fn test_as_slice() {
        let buffer = Buffer::new([1, 2, 3, 4, 5]);
        assert_eq!(buffer.as_slice(), Some(&[1, 2, 3, 4, 5][..]));
    }

    #[test]
    fn test_at() {
        let buffer = Buffer::new([1, 2, 3, 4, 5]);
        assert_eq!(buffer.at(0), Some(1));
        assert_eq!(buffer.at(4), Some(5));
        assert_eq!(buffer.at(5), None);
    }

    #[test]
    fn test_len() {
        let buffer = Buffer::new([1, 2, 3, 4, 5]);
        assert_eq!(buffer.len(), 5);
    }

    #[test]
    fn test_remove_prefix() {
        let mut buffer = Buffer::new([1, 2, 3, 4, 5]);
        assert_eq!(buffer.remove_prefix(3), Ok(()));
        assert_eq!(buffer.as_slice(), Some(&[4, 5][..]));
        assert_eq!(
            buffer.remove_prefix(3),
            Err("Buffer::remove_prefix n too large")
        );
    }

    #[test]
    fn test_as_ref() {
        let buffer = Buffer::new([1, 2, 3, 4, 5]);
        assert_eq!(buffer.as_ref(), &[1, 2, 3, 4, 5][..]);
    }

    // BufferList tests
    #[test]
    fn test_append() {
        let mut buffer_list1 = BufferList {
            buffers: VecDeque::from(vec![Buffer::new([1, 2, 3])]),
        };
        let mut buffer_list2 = BufferList {
            buffers: VecDeque::from(vec![Buffer::new([4, 5, 6])]),
        };
        buffer_list1.append(&mut buffer_list2);
        assert_eq!(buffer_list1.len(), 6);
        assert_eq!(buffer_list2.len(), 0);
    }

    #[test]
    fn test_len_buffer_list() {
        let buffer_list = BufferList {
            buffers: VecDeque::from(vec![Buffer::new([1, 2, 3]), Buffer::new([4, 5, 6])]),
        };
        assert_eq!(buffer_list.len(), 6);
    }

    #[test]
    fn test_buffers() {
        let buffer_list = BufferList {
            buffers: VecDeque::from(vec![Buffer::new([1, 2, 3]), Buffer::new([4, 5, 6])]),
        };
        assert_eq!(buffer_list.buffers().len(), 2);
    }

    #[test]
    fn test_to_buffer() {
        let buffer_list = BufferList {
            buffers: VecDeque::from(vec![Buffer::new([1, 2, 3])]),
        };
        assert_eq!(
            buffer_list.to_buffer().unwrap().unwrap().as_slice(),
            Some(&[1, 2, 3][..])
        );
    }

    #[test]
    fn test_remove_prefix_buffer_list() {
        let mut buffer_list = BufferList {
            buffers: VecDeque::from(vec![Buffer::new([1, 2, 3]), Buffer::new([4, 5, 6])]),
        };
        assert_eq!(buffer_list.remove_prefix(4), Ok(()));
        assert_eq!(buffer_list.len(), 2);
        assert_eq!(
            buffer_list.remove_prefix(3),
            Err("BufferList::remove_prefix out of bounds")
        );
    }

    #[test]
    fn test_concatenate() {
        let buffer_list = BufferList {
            buffers: VecDeque::from(vec![Buffer::new([1, 2, 3]), Buffer::new([4, 5, 6])]),
        };
        let result = buffer_list.concatenate();
        assert_eq!(result, [1, 2, 3, 4, 5, 6]);
    }

    // BufferViewList tests
    #[test]
    fn test_new_buffer_viewlist() {
        let buffer_list = BufferList {
            buffers: VecDeque::from(vec![Buffer::new([1, 2, 3]), Buffer::new([4, 5, 6])]),
        };
        let view_list = BufferViewList::new(&buffer_list);
        assert_eq!(view_list.len(), 6);
    }

    #[test]
    fn test_new_from_slice() {
        let slice = &[1, 2, 3, 4, 5, 6];
        let view_list = BufferViewList::new_frome_slice(slice);
        assert_eq!(view_list.len(), 6);
    }

    #[test]
    fn test_len_buffer_viewlist() {
        let slice = &[1, 2, 3, 4, 5, 6];
        let view_list = BufferViewList::new_frome_slice(slice);
        assert_eq!(view_list.len(), 6);
    }

    #[test]
    fn test_is_empty() {
        let slice = &[];
        let view_list = BufferViewList::new_frome_slice(slice);
        assert!(view_list.is_empty());
    }

    #[test]
    fn test_remove_prefix_buffer_viewlist() {
        let slice = &[1, 2, 3, 4, 5, 6];
        let mut view_list = BufferViewList::new_frome_slice(slice);
        assert_eq!(view_list.remove_prefix(3), Ok(()));
        assert_eq!(view_list.len(), 3);
        assert_eq!(
            view_list.remove_prefix(4),
            Err("BufferViewList::remove_prefix out of bounds")
        );
    }

    #[test]
    fn test_as_io_slices() {
        let slice = &[1, 2, 3, 4, 5, 6];
        let view_list = BufferViewList::new_frome_slice(slice);
        let io_slices = view_list.as_io_slices();
        assert_eq!(io_slices.len(), 1);
        assert_eq!(io_slices[0].len(), 6);
    }
}
