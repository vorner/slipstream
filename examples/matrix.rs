#![feature(test)]
extern crate test;

use std::ops::Mul;
use std::time::Instant;

use array_init::array_init;
use impatient::{InstructionSet, Polyfill, Sse4_1};
use rand::random;

const SIZE: usize = 512;
struct Matrix([[u32; SIZE]; SIZE]);

impl Matrix {
    fn random() -> Self {
        Self(array_init(|_| {
            array_init(|_| {
                random()
            })
        }))
    }

    #[inline]
    fn mult_simd<I: InstructionSet>(&self, is: I, rhs: &Matrix) -> Matrix {
        let mut output = [[0u32; SIZE]; SIZE];
        // TODO: This should need just one type parameter
        let mut column = [is.splat::<I::u32x16, _, _>(0); SIZE / 16];
        for x in 0..SIZE {
            // Do we want some kind of gather/stride way to load the vectors?
            // Anyway, as this is likely slower, we make sure to_mm_mullo_epu16 do the columns less often and
            // cache them for each corresponding rows, which load much faster
            for i in 0..SIZE {
                column[i / 16][i % 16] = rhs.0[i][x];
            }
            for y in 0..SIZE {
                let mut result: I::u32x16 = is.splat(0);
                for (c, r) in column.iter().zip(self.0[y].chunks_exact(16)) {
                    result += is.load::<I::u32x16, _, _>(r) * *c;
                }

                for p in result.iter() {
                    output[y][x] = output[y][x].wrapping_add(*p);
                }
            }
        }
        Matrix(output)
    }
}

impl Mul for &'_ Matrix {
    type Output = Matrix;
    fn mul(self, rhs: &Matrix) -> Matrix {
        let mut output = [[0u32; SIZE]; SIZE];
        for x in 0..SIZE {
            for y in 0..SIZE {
                for z in 0..SIZE {
                    output[y][x] = output[y][x].wrapping_add(self.0[y][z].wrapping_mul(rhs.0[z][x]));
                }
            }
        }
        Matrix(output)
    }
}

#[target_feature(enable = "sse2", enable = "sse4.1")]
unsafe fn mul_sse(sse: Sse4_1, lhs: &Matrix, rhs: &Matrix) -> Matrix {
    lhs.mult_simd(sse, rhs)
}

fn timed<R, F: FnOnce() -> R>(f: F) -> R {
    let now = Instant::now();
    let result = test::black_box(f());
    println!("took {:?}", now.elapsed());
    result
}

fn main() {
    let a = Matrix::random();
    let b = Matrix::random();
    let z = timed(|| &a * &b);
    let w = timed(|| a.mult_simd(Polyfill, &b));
    //assert_eq!(z, w);
    if let Ok(sse) = Sse4_1::detect() {
        let w = timed(|| unsafe { mul_sse(sse, &a, &b) });
        //assert_eq!(z, w);
    }
}
