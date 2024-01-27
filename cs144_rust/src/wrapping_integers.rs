use std::fmt;
use std::ops::{Add, Sub};

/// A 32-bit integer, expressed relative to an arbitrary initial sequence number (ISN)
/// This is used to express TCP sequence numbers (seqno) and acknowledgment numbers (ackno)
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct WrappingInt32(u32);

impl WrappingInt32 {
    /// Construct from a raw 32-bit unsigned integer
    pub fn new(raw_value: u32) -> Self {
        Self(raw_value)
    }

    /// Access raw stored value
    pub fn raw_value(self) -> u32 {
        self.0
    }

    /// Transform a 64-bit absolute sequence number (zero-indexed) into a 32-bit relative sequence number
    pub fn wrap(n: u64, isn: WrappingInt32) -> Self {
        WrappingInt32(((n & 0xFFFF_FFFF) as u32).wrapping_add(isn.0)) // wrapping_add 溢出取 mod
    }

    /// Transform a 32-bit relative sequence number into a 64-bit absolute sequence number (zero-indexed)
    /// fork form https://github.com/Kiprey/sponge
    pub fn unwrap(self, isn: WrappingInt32, checkpoint: u64) -> u64 {
        let offset = self.0.wrapping_sub(isn.0) as u32;

        if offset as u64 >= checkpoint {
            // 第一轮溢出都没有经过
            return offset as u64;
        }

        let real_checkpoint = checkpoint - offset as u64 + (1u64 << 31) as u64;
        let wrap_num = real_checkpoint / (1u64 << 32) as u64; // 获取最终的溢出次数

        return offset as u64 + wrap_num * (1u64 << 32) as u64;
    }
}

impl fmt::Display for WrappingInt32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn test_new() {
        let w = WrappingInt32::new(12345);
        assert_eq!(w.raw_value(), 12345);
    }

    #[test]
    fn test_wrap() {
        let mut rng = rand::thread_rng();
        for _ in 0..10000 {
            let isn_value = rng.gen_range(0..=std::u32::MAX);
            let isn = WrappingInt32::new(isn_value);
            let n = rng.gen_range(0..=std::u64::MAX);
            let w = WrappingInt32::wrap(n, isn);
            assert_eq!(
                w.raw_value(),
                ((n & 0xFFFF_FFFFu64) as u32).wrapping_add(isn_value)
            );
        }
    }

    #[test]
    fn test_wrap_unwrap() {
        let mut rng = rand::thread_rng();
        for _ in 0..10000 {
            let isn_value = rng.gen_range(0..=std::u32::MAX);
            let isn = WrappingInt32::new(isn_value);
            let n = rng.gen_range(0..=std::u64::MAX);
            let w = WrappingInt32::wrap(n, isn);
            let u = w.unwrap(isn, n & 0xFFFF_FFFF_FFFF_FFFF);
            assert_eq!(u, n & 0xFFFF_FFFF_FFFF_FFFF);
        }
    }

    #[test]
    fn test_display() {
        let w = WrappingInt32::new(12345);
        assert_eq!(format!("{}", w), "12345");
    }
}
