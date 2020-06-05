use multiversion::multiversion;
use test::Bencher;

use crate::mv;
use crate::utils::{gen_data, gen_vecs, V};
use splitstream::prelude::*;

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

    fn packed(l: &[f32], r: &[f32]) -> f32 {
        type V = packed_simd::f32x16;
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
        let result: f32 = l.iter()
            .zip(r.iter())
            .map(|(&l, &r)| {
                l * r
            })
            .sum();
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
#[cfg(target_arch = "x86_64")]
fn manual_sse(b: &mut Bencher) {
    use core::arch::x86_64 as arch;
    use std::mem;

    use crate::utils::gen_arch_vecs;

    let (l, r) = gen_arch_vecs();

    b.iter(|| {
        unsafe {
            let mut result = arch::_mm_setzero_ps();
            for (&l, &r) in l.iter().zip(r.iter()) {
                result = arch::_mm_add_ps(result, arch::_mm_mul_ps(l, r));
            }

            let result: [f32; 4] = mem::transmute(result);
            test::black_box(result.iter().sum::<f32>());
        }
    })
}

#[bench]
#[cfg(target_arch = "x86_64")]
fn manual_sse_fmadd(b: &mut Bencher) {
    use core::arch::x86_64::{self as arch, __m128};
    use std::mem;

    use crate::utils::gen_arch_vecs;

    let (l, r) = gen_arch_vecs();

    #[inline]
    #[target_feature(enable = "fma")]
    unsafe fn inner(l: &[__m128], r: &[__m128]) {
        let mut result = arch::_mm_setzero_ps();
        for (&l, &r) in l.iter().zip(r.iter()) {
            result = arch::_mm_fmadd_ps(l, r, result);
        }

        let result: [f32; 4] = mem::transmute(result);
        test::black_box(result.iter().sum::<f32>());
    }

    if is_x86_feature_detected!("fma") {
        b.iter(|| {
            unsafe {
                inner(l, r);
            }
        });
    }
}
