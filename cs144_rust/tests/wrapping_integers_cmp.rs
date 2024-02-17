use cs144_rust::wrapping_integers::WrappingInt32;
use rand::Rng;

#[test]
fn test_wrapping_int32_comparison() {
    // Comparing low-number adjacent seqnos
    assert_ne!(WrappingInt32::new(3), WrappingInt32::new(1));
    assert_eq!(WrappingInt32::new(3), WrappingInt32::new(3));

    const N_REPS: usize = 4096;

    let mut rng = rand::thread_rng();

    for _ in 0..N_REPS {
        let n: u32 = rng.gen();
        let diff: u8 = rng.gen();
        let m: u32 = n + diff as u32;

        assert_eq!(
            WrappingInt32::new(n) == WrappingInt32::new(m),
            n == m,
            "WrappingInt32 equality failed with n = {} and m = {}",
            n,
            m
        );
        assert_eq!(
            WrappingInt32::new(n) != WrappingInt32::new(m),
            n != m,
            "WrappingInt32 inequality failed with n = {} and m = {}",
            n,
            m
        );
    }
}
