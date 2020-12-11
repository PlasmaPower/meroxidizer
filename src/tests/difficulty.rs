use crate::utils::difficulty_to_max_hash;

#[test]
fn max_hash() {
    assert_eq!(
        hex::encode(difficulty_to_max_hash(0)),
        "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
    );
    assert_eq!(
        hex::encode(difficulty_to_max_hash(1)),
        "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
    );
    assert_eq!(
        hex::encode(difficulty_to_max_hash(2)),
        "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7f"
    );
    assert_eq!(
        hex::encode(difficulty_to_max_hash(3)),
        "5555555555555555555555555555555555555555555555555555555555555555"
    );
    assert_eq!(
        hex::encode(difficulty_to_max_hash(4)),
        "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff3f"
    );
}
