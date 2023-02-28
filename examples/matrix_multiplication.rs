//! Fair warning: There are many things to think about when optimizing SIMD
//! matrix multiplication, and we did most of them here in the interest of
//! showing the library's peak SIMD throughput on a nontrivial example.
//!
//! If you are relatively unfamiliar with SIMD in general or this library in
//! particular, you may want to start with the simpler `dot_product` example.

use multiversion::{multiversion, target::target_cfg_f};
use rand::random;
use slipstream::prelude::*;
use std::fmt::Display;
use std::hint::black_box;
use std::iter;
use std::ops::Mul;
use std::time::Instant;

// Size of the matrices that are being computed
//
// This small matrix size is chosen such that the working set fits in L1 cache,
// which means we don't have to implement cache blocking optimizations to achive
// compute-bound performance and show the optimal effect of SIMD.
//
// The matrix size should be divisible by `V::LANES * CHUNK_VECS`.
//
const SIZE: usize = 80;

// Number of output SIMD vectors we process together
//
// This should be greater than one for several reasons:
// - Most compute-oriented CPUs can process multiple independent streams of
//   arithmetic operations concurrently (e.g. current Intel and AMD CPUs can
//   process two independent FMAs per CPU cycle). If we only feed those with a
//   single stream of instructions that depend on each other, we lose perf.
// - It is the granularity at which we amortize non-arithmetic operations like
//   loop control code and scalar element broadcasting.
//
// The dot product example does both with and without parallel streams if you
// want to gauge their importance on a simpler operation.
//
// Do not tune it too high or you will run out of CPU registers!
//
const CHUNK_VECS: usize = 10;

// Vector type
type Scalar = f32;
type V = f32x8;

// Number of benchmark repetitions
const RUNS: u32 = 10_000;

// FIXME: Depending on how lucky you are with memory allocator lottery, you may
//        or may not get a vector that's properly aligned for SIMD processing.
//        Using a Vec<V> would be better from this perspective.
#[derive(Debug, PartialEq)]
struct Matrix(Vec<Scalar>);

impl Matrix {
    fn random() -> Self {
        Self(iter::repeat_with(random).take(SIZE * SIZE).collect())
    }
}

impl Mul for &'_ Matrix {
    type Output = Matrix;

    #[inline(never)]
    fn mul(self, rhs: &Matrix) -> Matrix {
        let mut out = vec![0.0; SIZE * SIZE];
        // The textbook algorithm: iterate over output and lhs rows...
        for (lhs_row, out_row) in self.0.chunks_exact(SIZE).zip(out.chunks_exact_mut(SIZE)) {
            // ...then over output elements and rhs columns...
            for (col, out_elem) in out_row.iter_mut().enumerate() {
                let rhs_col = rhs.0.iter().skip(col).step_by(SIZE);
                // ...and compute dot product of selected lhs row and rhs column
                for (lhs_elem, rhs_elem) in lhs_row.iter().zip(rhs_col) {
                    *out_elem += *lhs_elem * *rhs_elem;
                }
            }
        }
        Matrix(out)
    }
}

// SIMD algorithm with compile-time or run-time SIMD instruction set detection
macro_rules! generate_mat_mult {
    ($name:ident, $dispatcher:literal) => {
        #[inline(never)]
        #[multiversion(targets = "simd", dispatcher = $dispatcher)]
        fn $name(lhs: &Matrix, rhs: &Matrix) -> Matrix {
            // For SIMD and ILP reasons, we'll slice matrix rows into chunks of
            // a certain number of elements. For simplicity, we assume that this
            // chunk size divides the matrix row size evenly.
            const CHUNK_ELEMS: usize = CHUNK_VECS * V::LANES;
            assert_eq!(SIZE % CHUNK_ELEMS, 0);

            // Set up output buffer
            const NUM_ELEMS: usize = SIZE * SIZE;
            let mut out = vec![0.0; NUM_ELEMS];

            // Let the compiler know that input matrices are the same size
            let lhs = &lhs.0[..NUM_ELEMS];
            let rhs = &rhs.0[..NUM_ELEMS];

            // Iterate over output and lhs rows
            for (out_row, lhs_row) in out.chunks_exact_mut(SIZE).zip(lhs.chunks_exact(SIZE)) {
                // Chunk down output row into bits that fit in CPU registers
                for (chunk, out_chunk) in out_row.chunks_exact_mut(CHUNK_ELEMS).enumerate() {
                    // Set up output accumulators (compiler will keep them in registers)
                    let mut out_accs = [V::default(); CHUNK_VECS];

                    // Iterate over columns of lhs and rows of rhs, and within
                    // the selected rows of rhs, target the chunk that
                    // corresponds to the output chunk that we're generating
                    for (lhs_elem, rhs_chunk) in lhs_row.iter().zip(
                        rhs.chunks_exact(CHUNK_ELEMS)
                            .skip(chunk)
                            .step_by(SIZE / CHUNK_ELEMS),
                    ) {
                        // Turn active lhs element into a vector
                        let lhs_elem_vec = V::splat(*lhs_elem);

                        // Add contribution from rhs chunk to the accumulator
                        //
                        // FIXME: Must manually replicate the job of vectorize()
                        //        here because the implementation of vectorize
                        //        does not let the compiler know which slices
                        //        are equally sized, and in tight loops this is
                        //        very important.
                        //
                        for (out_acc, rhs_vec) in out_accs
                            .iter_mut()
                            .zip(rhs_chunk.chunks_exact(V::LANES).map(V::new))
                        {
                            if target_cfg_f!(target_feature = "fma") {
                                *out_acc = lhs_elem_vec.mul_add(rhs_vec, *out_acc);
                            } else {
                                *out_acc += lhs_elem_vec * rhs_vec;
                            }
                        }
                    }

                    // Spill output accumulators into output storage
                    //
                    // FIXME: Must manually replicate the job of vectorize()
                    //        here because the implementation of vectorize
                    //        does not let the compiler know which slices
                    //        are equally sized, and in tight loops this is
                    //        very important.
                    //
                    for (out_chunk, out_acc) in
                        out_chunk.chunks_exact_mut(V::LANES).zip(out_accs.iter())
                    {
                        out_acc.store(out_chunk);
                    }
                }
            }
            Matrix(out)
        }
    };
}
generate_mat_mult!(mat_mult_static, "static");
generate_mat_mult!(mat_mult_dynamic, "default");

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
    let a = Matrix::random();
    let b = Matrix::random();

    let m0 = timed("Scalar multiplication", || black_box(&a) * black_box(&b));
    let m1 = timed("Compile-time detected", || {
        mat_mult_static(black_box(&a), black_box(&b))
    });
    let m2 = timed("Run-time detected", || {
        mat_mult_dynamic(black_box(&a), black_box(&b))
    });

    let assert_close = |mref: &Matrix, mtest: &Matrix| {
        const TOLERANCE: Scalar = 1e-6;
        assert!(mref
            .0
            .iter()
            .zip(mtest.0.iter())
            .all(|(eref, etest)| (eref - etest).abs() < TOLERANCE * eref.abs()));
    };
    assert_close(&m0, &m1);
    assert_close(&m0, &m2);
}
