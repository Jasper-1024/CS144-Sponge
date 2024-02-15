use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io;
use std::result;

use rand::rngs::ThreadRng;
use std::fmt::Write;
use std::io::Write as IOWrite;
use std::net::Ipv4Addr;
use std::time::{Duration, Instant};

// 用于定义 Result 类型的别名，使其使用我们自定义的错误类型
type Result<T> = result::Result<T, Box<dyn Error>>;

// 实现一个标记错误类型，它包含了错误描述和相关的上下文信息
#[derive(Debug)]
struct TaggedError {
    // 尝试的操作描述
    attempt: String,
    // 实际发生的系统错误
    source: io::Error,
}

impl TaggedError {
    // 创建新的 TaggedError 实例
    fn new(attempt: &str, source: io::Error) -> Self {
        TaggedError {
            attempt: attempt.to_string(),
            source,
        }
    }
}

impl Display for TaggedError {
    // 实现 Display trait 用于友好地显示错误信息
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.attempt, self.source)
    }
}

impl Error for TaggedError {
    // 实现 Error trait，允许其他错误可以链式关联到这个错误上
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.source)
    }
}

// syscalls 的错误类型
#[derive(Debug)]
pub(crate) struct UnixError {
    tagged_error: TaggedError,
}

impl UnixError {
    // 创建新的 UnixError 实例
    pub(crate) fn new(attempt: &str) -> Self {
        UnixError {
            tagged_error: TaggedError::new(attempt, io::Error::last_os_error()),
        }
    }
}

impl Display for UnixError {
    // 实现 Display trait 用于友好地显示错误信息
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.tagged_error, f)
    }
}

impl Error for UnixError {
    // 实现 Error trait，允许其他错误可以链式关联到这个错误上
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.tagged_error.source)
    }
}

/// util fn

/// 处理 SystemCall 返回值, 查看系统调用错误信息
fn system_call(attempt: &str, return_value: i32, errno_mask: i32) -> Result<()> {
    if return_value >= 0 || io::Error::last_os_error().raw_os_error() == Some(errno_mask) {
        Ok(())
    } else {
        Err(Box::new(UnixError::new(attempt))) // need to check again
    }
}

/// 获取自程序启动以来的时间（毫秒）
fn timestamp_ms() -> u64 {
    let start = Instant::now();
    start.elapsed().as_millis() as u64
}

/// 获取一个随机数生成器
/// c++ 中使用的是 mt19937 算法, rust 中有 [mt19937] 但貌似只有 浮点数生成..
/// 看代码 这个函数几乎都用在了 测试中, 这里暂且使用 rust std
fn get_random_generator() -> ThreadRng {
    rand::thread_rng()
}

///  The internet checksum algorithm
pub struct InternetChecksum {
    sum: u32,
    parity: bool,
}

impl Default for InternetChecksum {
    fn default() -> Self {
        Self {
            sum: Default::default(),
            parity: Default::default(),
        }
    }
}
// ip 头部校验和 | tcp udp 校验和
///! For more information, see the [Wikipedia page](https://en.wikipedia.org/wiki/IPv4_header_checksum)
///! on the Internet checksum, and consult the [IP](\ref rfc::rfc791) and [TCP](\ref rfc::rfc793) RFCs.
impl InternetChecksum {
    pub fn new(initial_sum: u32) -> Self {
        InternetChecksum {
            sum: initial_sum,
            parity: false,
        }
    }

    pub fn add(&mut self, data: &[u8]) {
        for &byte in data {
            let val = if !self.parity {
                (byte as u32) << 8
            } else {
                byte as u32
            };
            self.sum += val;
            self.parity = !self.parity;
        }
    }

    pub fn value(&self) -> u16 {
        let mut ret = self.sum;
        while ret > 0xffff {
            ret = (ret >> 16) + (ret & 0xffff);
        }
        !(ret as u16)
    }
}

/// hex dump 任何数据
fn hexdump(data: &[u8], indent: usize) {
    let indent_string = " ".repeat(indent);
    let mut printed = 0;
    let mut pchars = String::new();
    for &byte in data {
        if printed % 16 == 0 {
            if !pchars.is_empty() {
                println!("    {}", pchars);
                pchars.clear();
            }
            print!("{}{:08x}:    ", indent_string, printed);
        } else if printed % 2 == 0 {
            print!(" ");
        }
        print!("{:02x}", byte);
        pchars.push(if byte.is_ascii_graphic() {
            byte as char
        } else {
            '.'
        });
        printed += 1;
    }
    let print_rem = (16 - (printed % 16)) % 16;
    println!("{}{}", "   ".repeat(print_rem), pchars);
}
