use cs144_rust::wrapping_integers::WrappingInt32;

#[test]
fn wrap_tests() {
    // Wrap the value which is exactly at the start of the 4th cycle (3 * 2^32),
    // expecting it to be equivalent to an initial sequence number (ISN) of 0
    assert_eq!(
        WrappingInt32::wrap(3 * (1u64 << 32), WrappingInt32::new(0)),
        WrappingInt32::new(0)
    );

    // Wrap a value (3 * 2^32 + 17) with an ISN of 15,
    // expecting the wrapped value to be 32 (17 + 15, under modulo arithmetic)
    assert_eq!(
        WrappingInt32::wrap(3 * (1u64 << 32) + 17, WrappingInt32::new(15)),
        WrappingInt32::new(32)
    );

    // Wrap a value (7 * 2^32 - 2) with an ISN of 15,
    // since the net addition is -2 + 15 = 13, expecting the wrapped value to be 13
    assert_eq!(
        WrappingInt32::wrap(7 * (1u64 << 32) - 2, WrappingInt32::new(15)),
        WrappingInt32::new(13)
    );
}
