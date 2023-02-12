use test::Bencher;

use crate::mv;
use crate::utils::{gen_data, gen_vecs, V};
use slipstream::prelude::*;

mv! {
    fn vectorized_idx(l: &[V], r: &[V]) -> f32 {
        assert_eq!(l.len(), r.len());
        let mut result = V::default();
        for i in 0..l.len() {
            result += l[i] * r[i];
        }

        result.horizontal_sum()
    }

    fn vectorized(l: &[V], r: &[V]) -> f32 {
        (l, r).vectorize()
            .map(|(l, r)| l * r)
            .sum::<V>()
            .horizontal_sum()
    }

    fn vectorize_zip(l: &[f32], r: &[f32]) -> f32 {
        let l = l.vectorize();
        let r = r.vectorize();
        l.zip(r)
            .map(|(l, r): (V, V)| l * r)
            .sum::<V>()
            .horizontal_sum()
    }

    fn vectorize_tuple(l: &[f32], r: &[f32]) -> f32 {
        (l, r).vectorize()
            .map(|(l, r): (V, V)| l * r)
            .sum::<V>()
            .horizontal_sum()
    }

    fn vectorize_tuple_for(l: &[f32], r: &[f32]) -> f32 {
        let mut result = V::default();
        for (l, r) in (l, r).vectorize() {
            let (l, r): (V, V) = (l, r);
            result += l * r;
        }
        result.horizontal_sum()
    }

    fn packed(l: &[f32], r: &[f32]) -> f32 {
        type V = packed_simd_2::f32x16;
        let l = l.chunks_exact(16);
        let r = r.chunks_exact(16);
        let mut result = V::default();
        for (l, r) in l.zip(r) {
            let l = V::from_slice_unaligned(l);
            let r = V::from_slice_unaligned(r);
            result = l.mul_adde(r, result);
        }
        result.sum()
    }
}

#[bench]
fn simple(b: &mut Bencher) {
    let (l, r) = gen_data();

    b.iter(|| {
        let result: f32 = l.iter().zip(r.iter()).map(|(&l, &r)| l * r).sum();
        test::black_box(result);
    });
}

#[bench]
fn vectorized_default(b: &mut Bencher) {
    let (l, r) = gen_vecs();
    b.iter(|| {
        test::black_box(vectorized_default_version(l, r));
    });
}

#[bench]
fn vectorized_detect(b: &mut Bencher) {
    let (l, r) = gen_vecs();
    b.iter(|| {
        test::black_box(vectorized(l, r));
    });
}

#[bench]
fn vectorized_idx_default(b: &mut Bencher) {
    let (l, r) = gen_vecs();
    b.iter(|| {
        test::black_box(vectorized_idx_default_version(l, r));
    });
}

#[bench]
fn vectorized_idx_detect(b: &mut Bencher) {
    let (l, r) = gen_vecs();
    b.iter(|| {
        test::black_box(vectorized_idx(l, r));
    });
}

#[bench]
fn vectorize_zip_default(b: &mut Bencher) {
    let (l, r) = gen_data();
    b.iter(|| {
        test::black_box(vectorize_zip_default_version(l, r));
    });
}

#[bench]
fn vectorize_zip_detect(b: &mut Bencher) {
    let (l, r) = gen_data();
    b.iter(|| {
        test::black_box(vectorize_zip(l, r));
    });
}

#[bench]
fn vectorize_tuple_default(b: &mut Bencher) {
    let (l, r) = gen_data();
    b.iter(|| {
        test::black_box(vectorize_tuple_default_version(l, r));
    });
}

#[bench]
fn vectorize_tuple_detect(b: &mut Bencher) {
    let (l, r) = gen_data();
    b.iter(|| {
        test::black_box(vectorize_tuple(l, r));
    });
}

#[bench]
fn packed_default(b: &mut Bencher) {
    let (l, r) = gen_data();
    b.iter(|| {
        test::black_box(packed_default_version(l, r));
    });
}

#[bench]
fn packed_detect(b: &mut Bencher) {
    let (l, r) = gen_data();
    b.iter(|| {
        test::black_box(packed(l, r));
    });
}

#[bench]
fn vectorize_tuple_for_default(b: &mut Bencher) {
    let (l, r) = gen_data();
    b.iter(|| {
        test::black_box(vectorize_tuple_for_default_version(l, r));
    });
}

#[bench]
fn vectorize_tuple_for_detect(b: &mut Bencher) {
    let (l, r) = gen_data();
    b.iter(|| {
        test::black_box(vectorize_tuple_for(l, r));
    });
}

#[bench]
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
fn manual_sse(b: &mut Bencher) {
    use std::mem;

    use crate::utils::arch::{self, __m128};
    use crate::utils::gen_arch_vecs;

    let (l, r) = gen_arch_vecs();

    #[target_feature(enable = "fma", enable = "sse")]
    unsafe fn inner(l: &[__m128], r: &[__m128]) -> f32 {
        let mut result = arch::_mm_setzero_ps();
        for (&l, &r) in l.iter().zip(r.iter()) {
            result = arch::_mm_add_ps(result, arch::_mm_mul_ps(l, r));
        }

        let result: [f32; 4] = mem::transmute(result);
        result.iter().sum()
    }

    b.iter(|| test::black_box(unsafe { inner(l, r) }));
}

#[bench]
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
fn manual_sse_fmadd(b: &mut Bencher) {
    use std::mem;

    use crate::utils::arch::{self, __m128};
    use crate::utils::gen_arch_vecs;

    let (l, r) = gen_arch_vecs();

    #[target_feature(enable = "fma", enable = "sse")]
    unsafe fn inner(l: &[__m128], r: &[__m128]) -> f32 {
        let mut result = arch::_mm_setzero_ps();
        for (&l, &r) in l.iter().zip(r.iter()) {
            result = arch::_mm_fmadd_ps(l, r, result);
        }

        let result: [f32; 4] = mem::transmute(result);
        result.iter().sum()
    }

    if is_x86_feature_detected!("fma") {
        b.iter(|| unsafe { test::black_box(inner(l, r)) });
    }
}
