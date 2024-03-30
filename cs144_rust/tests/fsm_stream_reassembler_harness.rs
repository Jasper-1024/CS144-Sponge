use cs144_rust::{
    byte_stream::ByteStreamTrait,
    stream_reassembler::{StreamReassembler, StreamReassemblerTrait},
};
#[allow(dead_code)]
pub(crate) struct ReassemblerTestHarness<'a> {
    stream_reassembler: StreamReassembler<'a>,
}

#[allow(dead_code)]
impl<'a, 'b> ReassemblerTestHarness<'a> {
    pub(crate) fn new(capacity: usize) -> Self {
        ReassemblerTestHarness {
            stream_reassembler: StreamReassembler::new(capacity).into(),
        }
    }

    pub(crate) fn submit_segment(&mut self, data: &'a [u8], index: u64) {
        self.stream_reassembler.push_substring(data, index, false);
    }

    pub(crate) fn submit_segment_default(&mut self, data: &'a [u8], index: u64, eof: bool) {
        self.stream_reassembler.push_substring(data, index, eof);
    }
    // 比较 byte_stream::bytes_written
    pub(crate) fn bytes_assembled(&self, bytes: usize) {
        let binding = self.stream_reassembler.stream_out();
        let byte_stream = binding.borrow_mut();
        let temp = byte_stream.bytes_written();
        assert_eq!(temp, bytes as u64);

        assert_eq!(
            temp, bytes as u64,
            "The reassembler was expected to have `{}` total bytes assembled, but there were `{}`",
            bytes, temp
        );
    }
    // bytes_stream::buffer_size  and bytes_stream::read
    pub(crate) fn bytes_available(&self, data: &'a [u8]) {
        let binding = self.stream_reassembler.stream_out();
        let mut byte_stream = binding.borrow_mut();

        let actual_size: usize = byte_stream.buffer_size();
        assert_eq!(
            actual_size,
            data.len(),
            "The reassembler was expected to have `{}` bytes available, but there were `{}`",
            data.len(),
            actual_size
        );

        let read = byte_stream.read(data.len());
        assert_eq!(
            read, data,
            "The reassembler was expected to have bytes \"{:?}\", but there were \"{:?}\"",
            data, read
        );
    }

    pub(crate) fn not_at_eof(&self) {
        let binding = self.stream_reassembler.stream_out();
        let byte_stream = binding.borrow_mut();

        assert!(
            !byte_stream.eof(),
            "The reassembler was expected to not be at EOF, but it was"
        );
    }

    pub(crate) fn at_eof(&self) {
        let binding = self.stream_reassembler.stream_out();
        let byte_stream = binding.borrow_mut();

        assert!(
            byte_stream.eof(),
            "The reassembler was expected to be at EOF, but it was"
        );
    }

    pub(crate) fn unassembled_bytes(&self, bytes: usize) {
        assert_eq!(
            self.stream_reassembler.unassembled_bytes(),
            bytes,
            "The reassembler was expected to have `{}` unassembled bytes, but there were `{}`",
            bytes,
            self.stream_reassembler.unassembled_bytes()
        );
    }
}
