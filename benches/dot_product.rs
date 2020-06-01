use std::iter;

use impatient::prelude::*;
use multiversion::multiversion;
use test::Bencher;

use crate::mv;
use crate::utils::{gen_arch_vecs, gen_data, gen_vecs, V};

mv! {
    fn vectorized(l: &[V], r: &[V]) -> f32 {
        l.iter()
            .zip(r.iter())
            .map(|(&l, &r)| l * r)
            .sum::<V>()
            // TODO: Horizontal sum
            .iter()
            .rev()
            .sum()
    }
}

#[bench]
fn simple(b: &mut Bencher) {
    let l = gen_data();
    let r = gen_data();

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
    let l = gen_vecs();
    let r = gen_vecs();
    b.iter(|| {
        test::black_box(vectorized_default_version(&l, &r));
    });
}

#[bench]
fn vectorized_detect(b: &mut Bencher) {
    let l = gen_vecs();
    let r = gen_vecs();
    b.iter(|| {
        test::black_box(vectorized(&l, &r));
    });
}

#[bench]
#[cfg(target_arch = "x86_64")]
fn manual_sse(b: &mut Bencher) {
    use core::arch::x86_64 as arch;
    use std::mem;

    let l = gen_arch_vecs();
    let r = gen_arch_vecs();

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
