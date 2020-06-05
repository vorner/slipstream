#![feature(test)]
extern crate test;

use std::iter;
use std::fmt::Display;
use std::num::Wrapping;
use std::ops::Mul;
use std::time::Instant;

use multiversion::{multiversion, target};
use rand::random;
use slipstream::prelude::*;

const SIZE: usize = 1024;
type V = wu32x8;
type O = usizex8;
const L: usize = V::LANES;

#[derive(Debug, PartialEq)]
struct Matrix(Vec<Wrapping<u32>>);

#[inline]
fn at(x: usize, y: usize) -> usize {
    y * SIZE + x
}

impl Matrix {
    fn random() -> Self {
        Self(iter::repeat_with(random).map(Wrapping).take(SIZE * SIZE).collect())
    }

    #[multiversion]
    #[clone(target = "[x86|x86_64]+sse+sse2+sse3+sse4.1+avx+avx2+fma")]
    #[clone(target = "[x86|x86_64]+sse+sse2+sse3+sse4.1+avx")]
    #[clone(target = "[x86|x86_64]+sse+sse2+sse3+sse4.1")]
    #[static_dispatch(fn = "dot_prod")]
    fn mult_simd(&self, rhs: &Matrix) -> Matrix {
        let mut output = vec![Wrapping(0); SIZE * SIZE];

        // Pre-compute offsets when gathering the column
        let mut column: [V; SIZE / L] = [Default::default(); SIZE / L];
        let offsets = (0..L).collect::<Vec<_>>();
        let base_offsets = O::new(offsets) * O::splat(SIZE);
        let mut offsets: [O; SIZE / L] = [Default::default(); SIZE / L];
        for i in 0..SIZE / L {
            offsets[i] = base_offsets + O::splat(i * L * SIZE);
        }

        for x in 0..SIZE { // Across columns
            // The gather_load is likely slower than just vectorizing the row, so we do this less
            // often and just once for each column instead of each time.
            let local_offsets = O::splat(x);
            for (col, off) in (&mut column[..], &offsets[..]).vectorize() {
                *col = V::gather_load(&rhs.0, off + local_offsets);
            }

            for y in 0..SIZE { // Across rows
                let row_start = at(0, y);
                output[at(x, y)] = dot_prod(&self.0[row_start..row_start + SIZE], &column);
            }
        }
        Matrix(output)
    }
}

#[multiversion]
#[specialize(target = "[x86|x86_64]+sse+sse2+sse3+sse4.1+avx+avx2+fma", fn = "dot_prod_avx", unsafe = true)]
#[clone(target = "[x86|x86_64]+sse+sse2+sse3+sse4.1+avx")]
#[clone(target = "[x86|x86_64]+sse+sse2+sse3+sse4.1")]
fn dot_prod(row: &[Wrapping<u32>], column: &[V]) -> Wrapping<u32> {
    (row, column)
        .vectorize()
        .map(|(r, c): (V, V)| r * c)
        .sum::<V>()
        .horizontal_sum()
}

#[target("[x86|x86_64]+sse+sse2+sse3+sse4.1+avx+avx2+fma")]
unsafe fn dot_prod_avx(row: &[Wrapping<u32>], column: &[V]) -> Wrapping<u32> {
    let mut result = V::default();
    for (r, c) in (row, column).vectorize() {
        let r: V = r;
        result += r * c;
    }
    result.horizontal_sum()
}

impl Mul for &'_ Matrix {
    type Output = Matrix;
    fn mul(self, rhs: &Matrix) -> Matrix {
        let mut output = vec![Wrapping(0); SIZE * SIZE];
        for x in 0..SIZE {
            for y in 0..SIZE {
                for z in 0..SIZE {
                    output[at(x, y)] += self.0[at(z, y)] * rhs.0[at(x, z)];
                }
            }
        }
        Matrix(output)
    }
}

fn timed<N: Display, R, F: FnOnce() -> R>(name: N, f: F) -> R {
    let now = Instant::now();
    let result = test::black_box(f());
    println!("{} took:\t{:?}", name, now.elapsed());
    result
}

fn main() {
    let a = Matrix::random();
    let b = Matrix::random();
    let z = timed("Scalar multiplication", || &a * &b);
    let x = timed("Compile-time detected", || a.mult_simd_default_version(&b));
    let w = timed("Run-time detected", || a.mult_simd(&b));
    assert_eq!(z, x);
    assert_eq!(z, w);
}
