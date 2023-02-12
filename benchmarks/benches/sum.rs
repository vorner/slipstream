use test::Bencher;

use slipstream::prelude::*;

use crate::mv;
use crate::utils::{gen_data, gen_vecs, V};

#[bench]
fn basic(b: &mut Bencher) {
    let (data, _) = gen_data();

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

    fn vectorized_horizontal(data: &[V]) -> f32 {
        let mut result = V::default();

        for v in data {
            result += *v;
        }

        result.horizontal_sum()
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

        sum_up(&result[..])
    }

    fn vectorize(data: &[f32]) -> f32 {
        let mut result = V::default();

        for v in data.vectorize() {
            result += v;
        }

        result.iter().rev().sum()
    }

    fn vectorize_horizontal(data: &[f32]) -> f32 {
        let mut result = V::default();

        for v in data.vectorize() {
            result += v;
        }

        result.horizontal_sum()
    }

    fn sum(data: &[V]) -> f32 {
        data.iter()
            .copied()
            .sum::<V>()
            .horizontal_sum()
    }

    fn sum_vectorize(data: &[f32]) -> f32 {
        data.vectorize()
            .sum::<V>()
            .horizontal_sum()
    }

    // Testing what happens performance wise if we get mutable iteration in play
    fn vectorize_mut(data: &mut [f32]) -> f32 {
        let mut result = V::default();

        for v in data.vectorize() {
            result += *v;
        }

        result.horizontal_sum()
    }

    fn vectorize_pad(data: &[f32]) -> f32 {
        data[1..].vectorize_pad(V::default())
            .sum::<V>()
            .horizontal_sum()
    }

    fn vectorize_split(data: &[f32]) -> f32 {
        let len = data.len();
        let rem = len % V::LANES;
        let main = data[..len - rem].vectorize().sum::<V>().horizontal_sum();
        let rem = data[len - rem..].iter().sum::<f32>();
        main + rem
    }
}

#[bench]
fn vectorized_default(b: &mut Bencher) {
    let (data, _) = gen_vecs();

    b.iter(|| {
        test::black_box(vectorized_default_version(data));
    })
}

#[bench]
fn vectorized_detect(b: &mut Bencher) {
    let (data, _) = gen_vecs();

    b.iter(|| {
        test::black_box(vectorized(data));
    })
}

#[bench]
fn vectorized_rev_default(b: &mut Bencher) {
    let (data, _) = gen_vecs();

    b.iter(|| {
        test::black_box(vectorized_rev_default_version(data));
    })
}

#[bench]
fn vectorized_rev_detect(b: &mut Bencher) {
    let (data, _) = gen_vecs();

    b.iter(|| {
        test::black_box(vectorized_rev(data));
    })
}

#[bench]
fn vectorized_tree_default(b: &mut Bencher) {
    let (data, _) = gen_vecs();

    b.iter(|| {
        test::black_box(vectorized_tree_default_version(data));
    })
}

#[bench]
fn vectorized_tree_detect(b: &mut Bencher) {
    let (data, _) = gen_vecs();

    b.iter(|| {
        test::black_box(vectorized_tree(data));
    })
}

#[bench]
fn vectorize_default(b: &mut Bencher) {
    let (data, _) = gen_data();

    b.iter(|| {
        test::black_box(vectorize_default_version(data));
    });
}

#[bench]
fn vectorize_detect(b: &mut Bencher) {
    let (data, _) = gen_data();

    b.iter(|| {
        test::black_box(vectorize(data));
    });
}

#[bench]
fn vectorize_horizontal_default(b: &mut Bencher) {
    let (data, _) = gen_data();

    b.iter(|| {
        test::black_box(vectorize_horizontal_default_version(data));
    });
}

#[bench]
fn vectorize_horizontal_detect(b: &mut Bencher) {
    let (data, _) = gen_data();

    b.iter(|| {
        test::black_box(vectorize_horizontal(data));
    });
}

#[bench]
fn sum_vectorize_default(b: &mut Bencher) {
    let (data, _) = gen_data();

    b.iter(|| {
        test::black_box(sum_vectorize_default_version(data));
    })
}

#[bench]
fn sum_vectorize_detect(b: &mut Bencher) {
    let (data, _) = gen_data();

    b.iter(|| {
        test::black_box(sum_vectorize(data));
    })
}

#[bench]
fn vectorize_mut_default(b: &mut Bencher) {
    let (data, _) = gen_data();
    let mut data = data.to_vec();

    b.iter(|| {
        test::black_box(vectorize_mut_default_version(&mut data));
    })
}

#[bench]
fn vectorize_mut_detect(b: &mut Bencher) {
    let (data, _) = gen_data();
    let mut data = data.to_vec();

    b.iter(|| {
        test::black_box(vectorize_mut(&mut data));
    })
}

#[bench]
fn sum_default(b: &mut Bencher) {
    let (data, _) = gen_vecs();

    b.iter(|| {
        test::black_box(sum_default_version(data));
    })
}

#[bench]
fn sum_detect(b: &mut Bencher) {
    let (data, _) = gen_vecs();

    b.iter(|| {
        test::black_box(sum(data));
    })
}

#[bench]
fn vectorized_horizontal_default(b: &mut Bencher) {
    let (data, _) = gen_vecs();

    b.iter(|| {
        test::black_box(vectorized_horizontal_default_version(data));
    })
}

#[bench]
fn vectorized_horizontal_detect(b: &mut Bencher) {
    let (data, _) = gen_vecs();

    b.iter(|| {
        test::black_box(vectorized_horizontal(data));
    })
}

#[bench]
fn vectorize_pad_default(b: &mut Bencher) {
    let (data, _) = gen_data();

    b.iter(|| {
        test::black_box(vectorize_pad_default_version(data));
    })
}

#[bench]
fn vectorize_pad_detect(b: &mut Bencher) {
    let (data, _) = gen_data();

    b.iter(|| {
        test::black_box(vectorize_pad(data));
    })
}

#[bench]
fn vectorize_split_default(b: &mut Bencher) {
    let (data, _) = gen_data();

    b.iter(|| {
        test::black_box(vectorize_split_default_version(data));
    })
}

#[bench]
fn vectorize_split_detect(b: &mut Bencher) {
    let (data, _) = gen_data();

    b.iter(|| {
        test::black_box(vectorize_split(data));
    })
}

#[bench]
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
fn manual_sse(b: &mut Bencher) {
    use core::mem;

    use crate::utils::arch::{self, __m128};
    use crate::utils::gen_arch_vecs;

    let (data, _) = gen_arch_vecs();

    // Note: this is technically not correct on the x86 target, we should check first, but who
    // cares in benchmarks.
    #[target_feature(enable = "sse")]
    unsafe fn inner(d: &[__m128]) -> f32 {
        let mut result = arch::_mm_setzero_ps();
        for v in d {
            result = arch::_mm_add_ps(result, *v);
        }

        let result: [f32; 4] = mem::transmute(result);
        result.iter().sum::<f32>()
    }

    b.iter(|| test::black_box(unsafe { inner(data) }))
}

#[bench]
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
fn manual_sse_convert(b: &mut Bencher) {
    use core::mem;

    use crate::utils::arch;

    let (data, _) = gen_data();

    // Note: this is technically not correct on the x86 target, we should check first, but who
    // cares in benchmarks.
    #[target_feature(enable = "sse")]
    unsafe fn inner(d: &[f32]) -> f32 {
        let mut result = arch::_mm_setzero_ps();
        let iter = d.chunks_exact(4);
        let remainder = iter.remainder().iter().sum::<f32>();
        for v in iter {
            result = arch::_mm_add_ps(result, arch::_mm_loadu_ps(v.as_ptr()));
        }

        let result: [f32; 4] = mem::transmute(result);
        result.iter().sum::<f32>() + remainder
    }

    b.iter(|| test::black_box(unsafe { inner(data) }))
}
