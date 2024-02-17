use cs144_rust::wrapping_integers::WrappingInt32;
use rand::Rng;

fn check_roundtrip(isn: WrappingInt32, value: u64, checkpoint: u64) {
    assert_eq!(
            WrappingInt32::unwrap(WrappingInt32::wrap(value, isn), isn, checkpoint),
            value,
            "Expected unwrap(wrap()) to recover same value, and it didn't!\n  where value = {}, isn = {}, and checkpoint = {}\n  (Difference between value and checkpoint is {}.)",
            value,
            isn.raw_value(),
            checkpoint,
            value as i64 - checkpoint as i64
        );
}

#[test]
fn test_wrapping_int32_roundtrip() {
    let mut rng = rand::thread_rng();

    for _ in 0..1_000_000 {
        let isn = WrappingInt32::new(rng.gen());
        let val: u64 = rng.gen();
        let offset: u64 = rng.gen_range(0..1 << 31);
        let big_offset: u64 = (1u64 << 31) - 1;

        check_roundtrip(isn, val, val);
        check_roundtrip(isn, val.wrapping_add(1), val);
        check_roundtrip(isn, val.wrapping_sub(1), val);
        check_roundtrip(isn, val.wrapping_add(offset), val);
        check_roundtrip(isn, val.wrapping_sub(offset), val);
        check_roundtrip(isn, val.wrapping_add(big_offset), val);
        check_roundtrip(isn, val.wrapping_sub(big_offset), val);
    }
}
