use num::BigUint;

pub fn difficulty_to_max_hash(mut difficulty: u64) -> [u8; 32] {
    if difficulty == 0 {
        difficulty = 1;
    }
    let mut num = BigUint::from(2u8).pow(32 * 8);
    num += BigUint::from(difficulty - 1);
    num /= difficulty;
    num -= BigUint::from(1u8);
    let bytes = num.to_bytes_le();
    let mut out = [0u8; 32];
    out[..bytes.len()].copy_from_slice(&bytes);
    out
}
