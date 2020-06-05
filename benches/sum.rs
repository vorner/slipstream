use multiversion::multiversion;
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

        sum_up(&result)
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
#[cfg(target_arch = "x86_64")]
fn manual_sse(b: &mut Bencher) {
    use core::arch::x86_64 as arch;
    use core::mem;

    use crate::utils::gen_arch_vecs;

    let (data, _) = gen_arch_vecs();

    b.iter(|| {
        unsafe {
            let mut result = arch::_mm_setzero_ps();
            for v in data {
                result = arch::_mm_add_ps(result, *v);
            }

            let result: [f32; 4] = mem::transmute(result);
            test::black_box(result.iter().sum::<f32>());
        }
    })
}
