use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hex_literal::hex;
use meroxidizer::bls::SecretKey;

fn bls_signing(c: &mut Criterion) {
    let secret_key = SecretKey::new(&hex!(
        "131f1303ca424d66ee051041322c0284b6a31f77916d204a875ecc42928f7501"
    ))
    .unwrap();
    let message = b"hello world";
    c.bench_function("signing", |b| {
        let secret_key = black_box(secret_key.clone());
        b.iter(|| secret_key.sign(black_box(message)));
    });
}

criterion_group!(benches, bls_signing);
criterion_main!(benches);
