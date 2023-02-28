use multiversion::{multiversion, target::target_cfg_f};
use rand::random;
use slipstream::prelude::*;
use std::fmt::Display;
use std::hint::black_box;
use std::iter;
use std::ops::Mul;
use std::time::Instant;

// Size of the vectors that are being multiplied
//
// This small vector size is chosen such that the working set fits in L1 cache,
// which is required to get optimal performance out of SIMD.
//
// The vector size should be divisible by `V::LANES * CHUNK_VECS`.
//
const SIZE: usize = 4096;

// Number of output SIMD vectors we process concurrently in the
// instruction-parallel version.
//
// Most compute-oriented CPUs can process multiple independent streams of
// arithmetic operations concurrently (e.g. current Intel and AMD CPUs can
// process two independent FMAs per CPU cycle). If we only feed those with a
// single stream of instructions that depend on each other, we lose perf, as
// demonstrated in this example.
//
// Do not tune this too high or you will run out of CPU registers!
//
// For simplicity, we assume it's a power of two.
//
const CHUNK_VECS: usize = 4;

// Scalar and vector type
type Scalar = f32;
type V = f32x8;

// Number of benchmark repetitions
const RUNS: u32 = 1_000_000;

// FIXME: Depending on how lucky you are with memory allocator lottery, you may
//        or may not get a vector that's properly aligned for SIMD processing.
//        Using a Vec<V> would be better from this perspective.
#[derive(Debug, PartialEq)]
struct Vector(Vec<Scalar>);

impl Vector {
    fn random() -> Self {
        Self(iter::repeat_with(random).take(SIZE).collect())
    }
}

impl Mul for &'_ Vector {
    type Output = Scalar;

    #[inline(never)]
    fn mul(self, rhs: &Vector) -> Scalar {
        // The textbook algorithm: sum of component products
        self.0
            .iter()
            .zip(rhs.0.iter())
            .fold(0.0, |acc, (&l, &r)| acc + l * r)
    }
}

/// Simple SIMD dot product without parallel instruction streams
macro_rules! generate_simple_dot {
    ($name:ident, $dispatcher:literal) => {
        #[inline(never)]
        #[multiversion(targets = "simd", dispatcher = $dispatcher)]
        fn $name(lhs: &Vector, rhs: &Vector) -> Scalar {
            // Set up a single SIMD accumulator
            let mut accumulator = V::default();

            // Iterate over SIMD vectors and compute sum of products
            for (lvec, rvec) in (&lhs.0[..], &rhs.0[..]).vectorize() {
                if target_cfg_f!(target_feature = "fma") {
                    accumulator = lvec.mul_add(rvec, accumulator);
                } else {
                    accumulator += lvec * rvec;
                }
            }

            // Reduce SIMD vector of result
            accumulator.horizontal_sum()
        }
    };
}
generate_simple_dot!(simple_dot_static, "static");
generate_simple_dot!(simple_dot_dynamic, "default");

/// More advanced SIMD dot product with parallel instruction streams
macro_rules! generate_parallel_dot {
    ($name:ident, $dispatcher:literal) => {
        #[inline(never)]
        #[multiversion(targets = "simd", dispatcher = $dispatcher)]
        fn $name(lhs: &Vector, rhs: &Vector) -> Scalar {
            // Set up one accumulator per instruction stream
            let mut accumulators = [V::default(); CHUNK_VECS];

            // Let the compiler know that input vectors are the same size
            let lhs = &lhs.0[..];
            let rhs = &rhs.0[..lhs.len()];

            // Work as in simple_dot, but with multiple SIMD accumulators
            // operating over larger chunks of elements...
            const CHUNK_ELEMS: usize = CHUNK_VECS * V::LANES;
            assert_eq!(lhs.len() % CHUNK_ELEMS, 0);
            for (lchunk, rchunk) in lhs
                .chunks_exact(CHUNK_ELEMS)
                .zip(rhs.chunks_exact(CHUNK_ELEMS))
            {
                // ...then over SIMD vectors inside the elements
                //
                // FIXME: Must manually replicate the job of vectorize() here
                //        because the implementation of vectorize does not let
                //        the compiler know which slices are equally sized, and
                //        in tight loops this is very important.
                //
                #[inline(always)]
                fn vectorize_slice(s: &[Scalar]) -> impl Iterator<Item = V> + '_ {
                    assert_eq!(s.len() % V::LANES, 0);
                    s.chunks_exact(V::LANES).map(V::new)
                }
                //
                #[inline(always)]
                fn vectorize_pair<'a>(
                    s1: &'a [Scalar],
                    s2: &'a [Scalar],
                ) -> impl Iterator<Item = (V, V)> + 'a {
                    vectorize_slice(s1).zip(vectorize_slice(s2))
                }
                //
                for (acc, (lvec, rvec)) in
                    accumulators.iter_mut().zip(vectorize_pair(lchunk, rchunk))
                {
                    if target_cfg_f!(target_feature = "fma") {
                        *acc = lvec.mul_add(rvec, *acc);
                    } else {
                        *acc += lvec * rvec;
                    }
                }
            }

            // Reduce SIMD accumulators with maximal parallelism
            assert!(CHUNK_VECS.is_power_of_two());
            let mut stride = CHUNK_VECS / 2;
            while stride > 0 {
                for i in 0..stride {
                    accumulators[i] += accumulators[i + stride];
                }
                stride /= 2;
            }

            // Reduce the final SIMD accumulator
            accumulators[0].horizontal_sum()
        }
    };
}
generate_parallel_dot!(parallel_dot_static, "static");
generate_parallel_dot!(parallel_dot_dynamic, "default");

fn timed<N: Display, R, F: FnMut() -> R>(name: N, mut f: F) -> R {
    let mut result = None;
    let start = Instant::now();
    for _ in 0..RUNS {
        result = Some(black_box(f()));
    }
    let elapsed = start.elapsed();
    println!("{} took:\t{:?} ({:?}/run)", name, elapsed, elapsed / RUNS);
    result.unwrap()
}

fn main() {
    let a = Vector::random();
    let b = Vector::random();

    let r0 = timed("Scalar dot product", || black_box(&a) * black_box(&b));

    let r1 = timed("Simple SIMD, compile-time detected", || {
        simple_dot_static(black_box(&a), black_box(&b))
    });
    let r2 = timed("Simple SIMD, run-time detected", || {
        simple_dot_dynamic(black_box(&a), black_box(&b))
    });

    let r3 = timed("Parallel SIMD, compile-time detected", || {
        parallel_dot_static(black_box(&a), black_box(&b))
    });
    let r4 = timed("Parallel SIMD, run-time detected", || {
        parallel_dot_dynamic(black_box(&a), black_box(&b))
    });

    fn assert_close(rref: &Scalar, rtest: &Scalar) {
        const TOLERANCE: f32 = 1e-5;
        assert!((rref - rtest).abs() < TOLERANCE * rref.abs());
    }

    assert_close(&r0, &r1);
    assert_close(&r0, &r2);
    assert_close(&r0, &r3);
    assert_close(&r0, &r4);
}
