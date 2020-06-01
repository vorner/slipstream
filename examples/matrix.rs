#![feature(test)]
extern crate test;

use std::ops::Mul;
use std::time::Instant;

use array_init::array_init;
use impatient::prelude::*;
use multiversion::multiversion;
use rand::random;

const SIZE: usize = 512;
struct Matrix([[V; SIZE / V::LANES]; SIZE]);

type V = wu32x16;
const L: usize = V::LANES;

impl Matrix {
    fn random() -> Self {
        Self(array_init(|_| {
            array_init(|_| {
                let inner: [u32; 16] = random();
                V::new(&inner)
            })
        }))
    }

    #[multiversion]
    #[clone(target = "[x86|x86_64]+sse+sse2+sse3+sse4.1+avx+avx2")]
    #[clone(target = "[x86|x86_64]+sse+sse2+sse3+sse4.1+avx")]
    #[clone(target = "[x86|x86_64]+sse+sse2+sse3+sse4.1")]
    fn mult_simd(&self, rhs: &Matrix) -> Matrix {
        let mut output = [[V::default(); SIZE / L]; SIZE];
        let mut column: [V; SIZE / L] = [Default::default(); SIZE / L];
        for x in 0..SIZE {
            // Do we want some kind of gather/stride way to load the vectors?
            // Anyway, as this is likely slower, we make sure to do the columns less often and
            // cache them for each corresponding rows, which load much faster
            for i in 0..SIZE {
                column[i / L][i % L] = rhs.0[i][x / L][x % L];
            }

            for y in 0..SIZE {
                let mut result = V::default();
                for (c, r) in column.iter().zip(self.0[y].iter()) {
                    result += *c * *r;
                }

                for l in result.iter() {
                    output[y][x / L][x % L] = output[y][x / L][x % L].wrapping_add(*l);
                }
            }
        }
        Matrix(output)
    }
}

impl Mul for &'_ Matrix {
    type Output = Matrix;
    fn mul(self, rhs: &Matrix) -> Matrix {
        let mut output = [[V::default(); SIZE / L]; SIZE];
        for x in 0..SIZE {
            for y in 0..SIZE {
                for z in 0..SIZE {
                    output[y][x / L][x % L] = output[y][x / L][x % L].wrapping_add(self.0[y][z / L][z % L].wrapping_mul(rhs.0[z][x / L][x % L]));
                }
            }
        }
        Matrix(output)
    }
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
    let x = timed(|| a.mult_simd_default_version(&b));
    let w = timed(|| a.mult_simd(&b));
    //assert_eq!(z, w);
    /*
    if let Ok(sse) = Sse4_1::detect() {
        let w = timed(|| unsafe { mul_sse(sse, &a, &b) });
        //assert_eq!(z, w);
    }
    if let Ok(avx) = Avx2::detect() {
        let w = timed(|| unsafe { mul_avx(avx, &a, &b) });
        //assert_eq!(z, w);
    }
    */
}
