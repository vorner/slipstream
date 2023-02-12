use std::iter;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use multiversion::multiversion;

use slipstream::prelude::*;

type V = f32x8;

const SIZE: usize = 4096 * 100;

#[multiversion(targets = "simd")]
fn sum(data: &[V]) -> f32 {
    data.iter().copied().sum::<V>().horizontal_sum()
}

fn sum_scalar(data: &[f32]) -> f32 {
    data.iter().copied().sum()
}

#[multiversion(targets = "simd")]
fn dot_product(l: &[f32], r: &[f32]) -> f32 {
    (l, r)
        .vectorize()
        .map(|(l, r): (V, V)| l * r)
        .sum::<V>()
        .horizontal_sum()
}

fn dot_product_scalar(l: &[f32], r: &[f32]) -> f32 {
    l.iter().zip(r).map(|(l, r)| l * r).sum()
}

fn benchmark(c: &mut Criterion) {
    let vecs = iter::repeat_with(rand::random)
        .map(|v: [f32; V::LANES]| V::new(&v))
        .take(SIZE / V::LANES)
        .collect::<Vec<_>>();

    let scalars_a = iter::repeat_with(rand::random)
        .take(SIZE)
        .collect::<Vec<_>>();

    let scalars_b = iter::repeat_with(rand::random)
        .take(SIZE)
        .collect::<Vec<_>>();

    c.bench_function("sum_vec", |b| {
        b.iter(|| black_box(sum(&vecs)));
    });

    c.bench_function("sum_scalar", |b| {
        b.iter(|| black_box(sum_scalar(&scalars_a)));
    });

    c.bench_function("dot_product_vec", |b| {
        b.iter(|| black_box(dot_product(&scalars_a, &scalars_b)));
    });

    c.bench_function("dot_product_scalar", |b| {
        b.iter(|| black_box(dot_product_scalar(&scalars_a, &scalars_b)));
    });
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
