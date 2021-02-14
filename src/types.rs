#![allow(missing_docs)]
//! Type aliases of the commonly used vector types.
//!
//! While the vector types are created from the [`Vector`] by setting the base type and length,
//! this is seldom done in downstream code. Instead, this module provides the commonly used types
//! as aliases, like [u16x8]. See the [crate introduction](crate) for further details about the
//! naming convention.
//!
//! All these types are also exported as part of the [`prelude`][crate::prelude].
use core::num::Wrapping;

pub use crate::mask::{m16, m32, m64, m8, msize};
use crate::vector::align::*;
use crate::vector::Vector;

pub type bx2 = Vector<Align2, bool, 2>;
pub type bx4 = Vector<Align4, bool, 4>;
pub type bx8 = Vector<Align8, bool, 8>;
pub type bx16 = Vector<Align16, bool, 16>;
pub type bx32 = Vector<Align32, bool, 32>;

pub type m8x2 = Vector<Align2, m8, 2>;
pub type m8x4 = Vector<Align4, m8, 4>;
pub type m8x8 = Vector<Align8, m8, 8>;
pub type m8x16 = Vector<Align16, m8, 16>;
pub type m8x32 = Vector<Align32, m8, 32>;

pub type m16x2 = Vector<Align4, m16, 2>;
pub type m16x4 = Vector<Align8, m16, 4>;
pub type m16x8 = Vector<Align16, m16, 8>;
pub type m16x16 = Vector<Align32, m16, 16>;

pub type m32x2 = Vector<Align8, m32, 2>;
pub type m32x4 = Vector<Align16, m32, 4>;
pub type m32x8 = Vector<Align32, m32, 8>;
pub type m32x16 = Vector<Align64, m32, 16>;

pub type m64x2 = Vector<Align16, m64, 2>;
pub type m64x4 = Vector<Align32, m64, 4>;
pub type m64x8 = Vector<Align64, m64, 8>;
pub type m64x16 = Vector<Align128, m64, 16>;

pub type u8x2 = Vector<Align2, u8, 2>;
pub type u8x4 = Vector<Align4, u8, 4>;
pub type u8x8 = Vector<Align8, u8, 8>;
pub type u8x16 = Vector<Align16, u8, 16>;
pub type u8x32 = Vector<Align32, u8, 32>;

pub type u16x2 = Vector<Align4, u16, 2>;
pub type u16x4 = Vector<Align8, u16, 4>;
pub type u16x8 = Vector<Align16, u16, 8>;
pub type u16x16 = Vector<Align32, u16, 16>;

pub type u32x2 = Vector<Align8, u32, 2>;
pub type u32x4 = Vector<Align16, u32, 4>;
pub type u32x8 = Vector<Align32, u32, 8>;
pub type u32x16 = Vector<Align64, u32, 16>;

pub type u64x2 = Vector<Align16, u64, 2>;
pub type u64x4 = Vector<Align32, u64, 4>;
pub type u64x8 = Vector<Align64, u64, 8>;
pub type u64x16 = Vector<Align128, u64, 16>;

pub type wu8x2 = Vector<Align2, Wrapping<u8>, 2>;
pub type wu8x4 = Vector<Align4, Wrapping<u8>, 4>;
pub type wu8x8 = Vector<Align8, Wrapping<u8>, 8>;
pub type wu8x16 = Vector<Align16, Wrapping<u8>, 16>;
pub type wu8x32 = Vector<Align32, Wrapping<u8>, 32>;

pub type wu16x2 = Vector<Align4, Wrapping<u16>, 2>;
pub type wu16x4 = Vector<Align8, Wrapping<u16>, 4>;
pub type wu16x8 = Vector<Align16, Wrapping<u16>, 8>;
pub type wu16x16 = Vector<Align32, Wrapping<u16>, 16>;

pub type wu32x2 = Vector<Align8, Wrapping<u32>, 2>;
pub type wu32x4 = Vector<Align16, Wrapping<u32>, 4>;
pub type wu32x8 = Vector<Align32, Wrapping<u32>, 8>;
pub type wu32x16 = Vector<Align64, Wrapping<u32>, 16>;

pub type wu64x2 = Vector<Align16, Wrapping<u64>, 2>;
pub type wu64x4 = Vector<Align32, Wrapping<u64>, 4>;
pub type wu64x8 = Vector<Align64, Wrapping<u64>, 8>;
pub type wu64x16 = Vector<Align128, Wrapping<u64>, 16>;

pub type i8x2 = Vector<Align2, i8, 2>;
pub type i8x4 = Vector<Align4, i8, 4>;
pub type i8x8 = Vector<Align8, i8, 8>;
pub type i8x16 = Vector<Align16, i8, 16>;
pub type i8x32 = Vector<Align32, i8, 32>;

pub type i16x2 = Vector<Align4, i16, 2>;
pub type i16x4 = Vector<Align8, i16, 4>;
pub type i16x8 = Vector<Align16, i16, 8>;
pub type i16x16 = Vector<Align32, i16, 16>;

pub type i32x2 = Vector<Align8, i32, 2>;
pub type i32x4 = Vector<Align16, i32, 4>;
pub type i32x8 = Vector<Align32, i32, 8>;
pub type i32x16 = Vector<Align64, i32, 16>;

pub type i64x2 = Vector<Align16, i64, 2>;
pub type i64x4 = Vector<Align32, i64, 4>;
pub type i64x8 = Vector<Align64, i64, 8>;
pub type i64x16 = Vector<Align128, i64, 16>;

pub type wi8x2 = Vector<Align2, Wrapping<i8>, 2>;
pub type wi8x4 = Vector<Align4, Wrapping<i8>, 4>;
pub type wi8x8 = Vector<Align8, Wrapping<i8>, 8>;
pub type wi8x16 = Vector<Align16, Wrapping<i8>, 16>;
pub type wi8x32 = Vector<Align32, Wrapping<i8>, 32>;

pub type wi16x2 = Vector<Align4, Wrapping<i16>, 2>;
pub type wi16x4 = Vector<Align8, Wrapping<i16>, 4>;
pub type wi16x8 = Vector<Align16, Wrapping<i16>, 8>;
pub type wi16x16 = Vector<Align32, Wrapping<i16>, 16>;

pub type wi32x2 = Vector<Align8, Wrapping<i32>, 2>;
pub type wi32x4 = Vector<Align16, Wrapping<i32>, 4>;
pub type wi32x8 = Vector<Align32, Wrapping<i32>, 8>;
pub type wi32x16 = Vector<Align64, Wrapping<i32>, 16>;

pub type wi64x2 = Vector<Align16, Wrapping<i64>, 2>;
pub type wi64x4 = Vector<Align32, Wrapping<i64>, 4>;
pub type wi64x8 = Vector<Align64, Wrapping<i64>, 8>;
pub type wi64x16 = Vector<Align128, Wrapping<i64>, 16>;

pub type f32x2 = Vector<Align8, f32, 2>;
pub type f32x4 = Vector<Align16, f32, 4>;
pub type f32x8 = Vector<Align32, f32, 8>;
pub type f32x16 = Vector<Align64, f32, 16>;

pub type f64x2 = Vector<Align16, f64, 2>;
pub type f64x4 = Vector<Align32, f64, 4>;
pub type f64x8 = Vector<Align64, f64, 8>;
pub type f64x16 = Vector<Align128, f64, 16>;

// Note: the usize/isize vectors are per-pointer-width because they need a different alignment.

#[cfg(target_pointer_width = "32")]
mod sized {
    use super::*;

    pub type msizex2 = Vector<Align8, msize, 2>;
    pub type msizex4 = Vector<Align16, msize, 4>;
    pub type msizex8 = Vector<Align32, msize, 8>;
    pub type msizex16 = Vector<Align64, msize, 16>;

    pub type usizex2 = Vector<Align8, usize, 2>;
    pub type usizex4 = Vector<Align16, usize, 4>;
    pub type usizex8 = Vector<Align32, usize, 8>;
    pub type usizex16 = Vector<Align64, usize, 16>;

    pub type wusizex2 = Vector<Align8, Wrapping<usize>, 2>;
    pub type wusizex4 = Vector<Align16, Wrapping<usize>, 4>;
    pub type wusizex8 = Vector<Align32, Wrapping<usize>, 8>;
    pub type wusizex16 = Vector<Align64, Wrapping<usize>, 16>;

    pub type isizex2 = Vector<Align8, isize, 2>;
    pub type isizex4 = Vector<Align16, isize, 4>;
    pub type isizex8 = Vector<Align32, isize, 8>;
    pub type isizex16 = Vector<Align64, isize, 16>;

    pub type wisizex2 = Vector<Align8, Wrapping<isize>, 2>;
    pub type wisizex4 = Vector<Align16, Wrapping<isize>, 4>;
    pub type wisizex8 = Vector<Align32, Wrapping<isize>, 8>;
    pub type wisizex16 = Vector<Align64, Wrapping<isize>, 16>;
}

#[cfg(target_pointer_width = "64")]
mod sized {
    use super::*;

    pub type msizex2 = Vector<Align16, msize, 2>;
    pub type msizex4 = Vector<Align32, msize, 4>;
    pub type msizex8 = Vector<Align64, msize, 8>;
    pub type msizex16 = Vector<Align128, msize, 16>;

    pub type usizex2 = Vector<Align16, usize, 2>;
    pub type usizex4 = Vector<Align32, usize, 4>;
    pub type usizex8 = Vector<Align64, usize, 8>;
    pub type usizex16 = Vector<Align128, usize, 16>;

    pub type wusizex2 = Vector<Align16, Wrapping<usize>, 2>;
    pub type wusizex4 = Vector<Align32, Wrapping<usize>, 4>;
    pub type wusizex8 = Vector<Align64, Wrapping<usize>, 8>;
    pub type wusizex16 = Vector<Align128, Wrapping<usize>, 16>;

    pub type isizex2 = Vector<Align16, isize, 2>;
    pub type isizex4 = Vector<Align32, isize, 4>;
    pub type isizex8 = Vector<Align64, isize, 8>;
    pub type isizex16 = Vector<Align128, isize, 16>;

    pub type wisizex2 = Vector<Align16, Wrapping<isize>, 2>;
    pub type wisizex4 = Vector<Align32, Wrapping<isize>, 4>;
    pub type wisizex8 = Vector<Align64, Wrapping<isize>, 8>;
    pub type wisizex16 = Vector<Align128, Wrapping<isize>, 16>;
}

pub use sized::*;
