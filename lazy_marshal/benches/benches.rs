use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};
use lazy_marshal::prelude::*;

#[derive(Marshal, UnMarshal, Default, Clone, Copy)]
struct TestStruct {
    a: f32,
    b: f64,
    c: i8,
    d: u16,
    e: bool,
}

#[inline(never)]
fn marshal_u64(num: &str) -> u8 {
    unsafe { num.marshal().next().unwrap_unchecked() }
}

fn criterion_benchmark(c: &mut Criterion) {
    let s1 = TestStruct::default();

    c.bench_function("marshal_u8", |b| {
        b.iter(|| black_box(marshal_u64(black_box(Default::default()))))
    });
    // c.bench_function("Marshalling", |b| b.iter(|| black_box(s1).marshal()));

    // c.bench_function("UnMarshalling", |b| {
    //     b.iter(|| {
    //         let mut b = s1.marshal();
    //         TestStruct::unmarshal(black_box(&mut b)).unwrap();
    //     })
    // });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
