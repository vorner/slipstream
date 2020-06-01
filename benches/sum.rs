use std::iter;

use impatient::prelude::*;
use multiversion::multiversion;
use test::Bencher;

use crate::mv;
use crate::utils::{gen_data, gen_vecs, gen_arch_vecs, V, SIZE};

#[bench]
fn basic(b: &mut Bencher) {
    let data = gen_data();

    b.iter(|| {
        test::black_box(data.iter().sum::<f32>());
    })
}

mv! {
    fn vectorized(data: &[V]) -> f32 {
        let mut result = V::default();

        for v in data {
            result += *v;
        }

        result.iter().sum()
    }

    fn vectorized_rev(data: &[V]) -> f32 {
        let mut result = V::default();

        for v in data {
            result += *v;
        }

        // Any idea why this rev makes it run faster?
        result.iter().rev().sum()
    }

    fn vectorized_tree(data: &[V]) -> f32 {
        let mut result = V::default();

        for v in data {
            result += *v;
        }

        #[inline]
        fn sum_up(d: &[f32]) -> f32 {
            if d.len() == 1 {
                d[0]
            } else {
                let mid = d.len() / 2;
                sum_up(&d[..mid]) + sum_up(&d[mid..])
            }
        }

        sum_up(&result)
    }

    fn vectorize(data: &[f32]) -> f32 {
        let mut result = V::default();

        for v in impatient::vectorize_exact(data) {
            result += v;
        }

        result.iter().rev().sum()
    }

    fn sum(data: &[V]) -> f32 {
        data.iter()
            .copied()
            .sum::<V>()
            .iter()
            .rev()
            .sum()
    }

    fn sum_vectorize(data: &[f32]) -> f32 {
        impatient::vectorize_exact(data)
            .sum::<V>()
            .iter()
            .rev()
            .sum()
    }
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
fn vectorized_tree_default(b: &mut Bencher) {
    let data = gen_vecs();

    b.iter(|| {
        test::black_box(vectorized_tree_default_version(&data));
    })
}

#[bench]
fn vectorized_tree_detect(b: &mut Bencher) {
    let data = gen_vecs();

    b.iter(|| {
        test::black_box(vectorized_tree(&data));
    })
}

#[bench]
fn vectorize_default(b: &mut Bencher) {
    let data = gen_data();

    b.iter(|| {
        test::black_box(vectorize_default_version(&data));
    });
}

#[bench]
fn vectorize_detect(b: &mut Bencher) {
    let data = gen_data();

    b.iter(|| {
        test::black_box(vectorize(&data));
    });
}

#[bench]
fn sum_vectorize_default(b: &mut Bencher) {
    let data = gen_data();

    b.iter(|| {
        test::black_box(sum_vectorize_default_version(&data));
    })
}

#[bench]
fn sum_vectorize_detect(b: &mut Bencher) {
    let data = gen_data();

    b.iter(|| {
        test::black_box(sum_vectorize(&data));
    })
}


#[bench]
fn sum_default(b: &mut Bencher) {
    let data = gen_vecs();

    b.iter(|| {
        test::black_box(sum_default_version(&data));
    })
}

#[bench]
fn sum_detect(b: &mut Bencher) {
    let data = gen_vecs();

    b.iter(|| {
        test::black_box(sum(&data));
    })
}

#[bench]
#[cfg(target_arch = "x86_64")]
fn vectorized_manual_sse(b: &mut Bencher) {
    use core::arch::x86_64 as arch;
    use core::mem;

    let data = gen_arch_vecs();

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
