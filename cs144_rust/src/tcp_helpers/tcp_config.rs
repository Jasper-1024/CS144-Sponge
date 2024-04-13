use crate::{util::address::Address, wrapping_integers::WrappingInt32};

pub const DEFAULT_CAPACITY: usize = 64000; // 默认容量
pub const MAX_PAYLOAD_SIZE: usize = 1452; // 适合在IPv4或UDP数据报中的最大TCP负载
pub const TIMEOUT_DFLT: u16 = 1000; // 默认的重传超时时间为 1 秒 1000ms
pub const MAX_RETX_ATTEMPTS: usize = 8; // 放弃之前的最大重传尝试次数

/// Config for TCP sender and receiver
#[allow(dead_code)]
pub struct TCPConfig {
    pub rt_timeout: u16,                  // 重传超时的初始值，以毫秒为单位
    pub recv_capacity: usize,             // 接收容量，以字节为单位
    pub send_capacity: usize,             // 发送容量，以字节为单位
    pub fixed_isn: Option<WrappingInt32>, // 可选的固定初始序列号
}

impl TCPConfig {
    pub fn new() -> TCPConfig {
        TCPConfig {
            rt_timeout: TIMEOUT_DFLT,
            recv_capacity: DEFAULT_CAPACITY,
            send_capacity: DEFAULT_CAPACITY,
            fixed_isn: None,
        }
    }
}

impl Default for TCPConfig {
    fn default() -> Self {
        TCPConfig::new()
    }
}

/// Config for classes derived from FdAdapter
#[allow(unused)]
struct FdAdapterConfig {
    source: Address,      // source address and port
    destination: Address, // destination address and port
    loss_rate_dn: u16,    // loss rate for downlink (for LossyFdAdapter)
    loss_rate_up: u16,    // loss rate for uplink (for LossyFdAdapter)
}

impl Default for FdAdapterConfig {
    fn default() -> Self {
        FdAdapterConfig {
            source: Address::new_ip_port("0", 0).unwrap(),
            destination: Address::new_ip_port("0", 0).unwrap(),
            loss_rate_dn: 0,
            loss_rate_up: 0,
        }
    }
}
