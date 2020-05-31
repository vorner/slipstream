#![feature(test)]

extern crate test;

use std::iter;

use impatient::prelude::*;
use multiversion::multiversion;
use test::Bencher;

const SIZE: usize = 10*1024*1024;
type V = impatient::f32x16;

#[bench]
fn basic(b: &mut Bencher) {
    let data: Vec<f32> = iter::repeat_with(rand::random)
        .take(SIZE)
        .collect();

    b.iter(|| {
        test::black_box(data.iter().sum::<f32>());
    })
}

#[multiversion]
#[clone(target = "[x86|x86_64]+sse+sse2+sse3+sse4.1+avx+avx2")]
#[clone(target = "[x86|x86_64]+sse+sse2+sse3+sse4.1+avx")]
#[clone(target = "[x86|x86_64]+sse+sse2+sse3+sse4.1")]
fn vectorized(data: &[V]) -> f32 {
    let mut result = V::default();

    for v in data {
        result += *v;
    }

    result.iter().sum()
}

#[multiversion]
#[clone(target = "[x86|x86_64]+sse+sse2+sse3+sse4.1+avx+avx2")]
#[clone(target = "[x86|x86_64]+sse+sse2+sse3+sse4.1+avx")]
#[clone(target = "[x86|x86_64]+sse+sse2+sse3+sse4.1")]
fn vectorized_rev(data: &[V]) -> f32 {
    let mut result = V::default();

    for v in data {
        result += *v;
    }

    // Any idea why this rev makes it run faster?
    result.iter().rev().sum()
}

fn gen_vecs() -> Vec<V> {
    iter::repeat_with(rand::random)
        .map(|v: [f32; V::LANES]| V::new(&v))
        .take(SIZE / V::LANES)
        .collect()
}

#[bench]
fn vectorized_default(b: &mut Bencher) {
    let data = gen_vecs();

    b.iter(|| {
        test::black_box(vectorized_default_version(&data));
    })
}

#[bench]
fn vectorized_detect(b: &mut Bencher) {
    let data = gen_vecs();

    b.iter(|| {
        test::black_box(vectorized(&data));
    })
}

#[bench]
fn vectorized_rev_default(b: &mut Bencher) {
    let data = gen_vecs();

    b.iter(|| {
        test::black_box(vectorized_rev_default_version(&data));
    })
}

#[bench]
fn vectorized_rev_detect(b: &mut Bencher) {
    let data = gen_vecs();

    b.iter(|| {
        test::black_box(vectorized_rev(&data));
    })
}

#[bench]
#[cfg(target_arch = "x86_64")]
fn vectorized_manual_sse(b: &mut Bencher) {
    use core::arch::x86_64::{self as arch, __m128};
    use core::mem;

    let data: Vec<__m128>;
    unsafe {
        data = iter::repeat_with(|| {
                let v: [f32; 4] = rand::random();
                arch::_mm_loadu_ps(v.as_ptr())
            })
            .take(SIZE / 4)
            .collect();
    }

    b.iter(|| {
        unsafe {
            let mut result = arch::_mm_setzero_ps();
            for v in &data {
                result = arch::_mm_add_ps(result, *v);
            }

            let result: [f32; 4] = mem::transmute(result);
            test::black_box(result.iter().sum::<f32>());
        }
    })
}
