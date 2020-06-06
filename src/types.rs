use core::num::Wrapping;

use typenum::consts::*;

pub use crate::mask::{m16, m32, m64, m8, msize};
use crate::vector::*;

pub type bx2 = Packed2<bool, U2>;
pub type bx4 = Packed4<bool, U4>;
pub type bx8 = Packed8<bool, U8>;
pub type bx16 = Packed16<bool, U16>;
pub type bx32 = Packed32<bool, U32>;

pub type m8x2 = Packed2<m8, U2>;
pub type m8x4 = Packed4<m8, U4>;
pub type m8x8 = Packed8<m8, U8>;
pub type m8x16 = Packed16<m8, U16>;
pub type m8x32 = Packed32<m8, U32>;

pub type m16x2 = Packed4<m16, U2>;
pub type m16x4 = Packed8<m16, U4>;
pub type m16x8 = Packed16<m16, U8>;
pub type m16x16 = Packed32<m16, U16>;

pub type m32x2 = Packed8<m32, U2>;
pub type m32x4 = Packed16<m32, U4>;
pub type m32x8 = Packed32<m32, U8>;
pub type m32x16 = Packed32<m32, U16>;

pub type m64x2 = Packed16<m64, U2>;
pub type m64x4 = Packed32<m64, U4>;
pub type m64x8 = Packed32<m64, U8>;
pub type m64x16 = Packed32<m64, U16>;

pub type u8x2 = Packed2<u8, U2>;
pub type u8x4 = Packed4<u8, U4>;
pub type u8x8 = Packed8<u8, U8>;
pub type u8x16 = Packed16<u8, U16>;
pub type u8x32 = Packed16<u8, U32>;

pub type u16x2 = Packed4<u16, U2>;
pub type u16x4 = Packed8<u16, U4>;
pub type u16x8 = Packed16<u16, U8>;
pub type u16x16 = Packed32<u16, U16>;

pub type u32x2 = Packed8<u32, U2>;
pub type u32x4 = Packed16<u32, U4>;
pub type u32x8 = Packed32<u32, U8>;
pub type u32x16 = Packed32<u32, U16>;

pub type u64x2 = Packed16<u64, U2>;
pub type u64x4 = Packed32<u64, U4>;
pub type u64x8 = Packed32<u64, U8>;
pub type u64x16 = Packed32<u64, U16>;

pub type wu8x2 = Packed2<Wrapping<u8>, U2>;
pub type wu8x4 = Packed4<Wrapping<u8>, U4>;
pub type wu8x8 = Packed8<Wrapping<u8>, U8>;
pub type wu8x16 = Packed16<Wrapping<u8>, U16>;
pub type wu8x32 = Packed16<Wrapping<u8>, U32>;

pub type wu16x2 = Packed4<Wrapping<u16>, U2>;
pub type wu16x4 = Packed8<Wrapping<u16>, U4>;
pub type wu16x8 = Packed16<Wrapping<u16>, U8>;
pub type wu16x16 = Packed32<Wrapping<u16>, U16>;

pub type wu32x2 = Packed8<Wrapping<u32>, U2>;
pub type wu32x4 = Packed16<Wrapping<u32>, U4>;
pub type wu32x8 = Packed32<Wrapping<u32>, U8>;
pub type wu32x16 = Packed32<Wrapping<u32>, U16>;

pub type wu64x2 = Packed16<Wrapping<u64>, U2>;
pub type wu64x4 = Packed32<Wrapping<u64>, U4>;
pub type wu64x8 = Packed32<Wrapping<u64>, U8>;
pub type wu64x16 = Packed32<Wrapping<u64>, U16>;

// TODO: u/i not wrapping
// TODO: usize vectors ‒ alignments based on arch

pub type i8x2 = Packed2<i8, U2>;
pub type i8x4 = Packed4<i8, U4>;
pub type i8x8 = Packed8<i8, U8>;
pub type i8x16 = Packed16<i8, U16>;

pub type i16x2 = Packed4<i16, U2>;
pub type i16x4 = Packed8<i16, U4>;
pub type i16x8 = Packed16<i16, U8>;
pub type i16x16 = Packed32<i16, U16>;

pub type i32x2 = Packed8<i32, U2>;
pub type i32x4 = Packed16<i32, U4>;
pub type i32x8 = Packed32<i32, U8>;
pub type i32x16 = Packed32<i32, U16>;

pub type i64x2 = Packed16<i64, U2>;
pub type i64x4 = Packed32<i64, U4>;
pub type i64x8 = Packed32<i64, U8>;
pub type i64x16 = Packed32<i64, U16>;

pub type wi8x2 = Packed2<Wrapping<i8>, U2>;
pub type wi8x4 = Packed4<Wrapping<i8>, U4>;
pub type wi8x8 = Packed8<Wrapping<i8>, U8>;
pub type wi8x16 = Packed16<Wrapping<i8>, U16>;

pub type wi16x2 = Packed4<Wrapping<i16>, U2>;
pub type wi16x4 = Packed8<Wrapping<i16>, U4>;
pub type wi16x8 = Packed16<Wrapping<i16>, U8>;
pub type wi16x16 = Packed32<Wrapping<i16>, U16>;

pub type wi32x2 = Packed8<Wrapping<i32>, U2>;
pub type wi32x4 = Packed16<Wrapping<i32>, U4>;
pub type wi32x8 = Packed32<Wrapping<i32>, U8>;
pub type wi32x16 = Packed32<Wrapping<i32>, U16>;

pub type wi64x2 = Packed16<Wrapping<i64>, U2>;
pub type wi64x4 = Packed32<Wrapping<i64>, U4>;
pub type wi64x8 = Packed32<Wrapping<i64>, U8>;
pub type wi64x16 = Packed32<Wrapping<i64>, U16>;

pub type f32x2 = Packed8<f32, U2>;
pub type f32x4 = Packed16<f32, U4>;
pub type f32x8 = Packed32<f32, U8>;
pub type f32x16 = Packed32<f32, U16>;

pub type f64x2 = Packed16<f64, U2>;
pub type f64x4 = Packed32<f64, U4>;
pub type f64x8 = Packed32<f64, U8>;
pub type f64x16 = Packed32<f64, U16>;

// Note: the usize/isize vectors are per-pointer-width because they need a different alignment.

#[cfg(target_pointer_width = "32")]
mod sized {
    use super::*;

    pub type msizex2 = Packed8<msize, U2>;
    pub type msizex4 = Packed16<msize, U4>;
    pub type msizex8 = Packed32<msize, U8>;
    pub type msizex16 = Packed32<msize, U16>;

    pub type usizex2 = Packed8<usize, U2>;
    pub type usizex4 = Packed16<usize, U4>;
    pub type usizex8 = Packed32<usize, U8>;
    pub type usizex16 = Packed32<usize, U16>;

    pub type wusizex2 = Packed8<Wrapping<usize>, U2>;
    pub type wusizex4 = Packed16<Wrapping<usize>, U4>;
    pub type wusizex8 = Packed32<Wrapping<usize>, U8>;
    pub type wusizex16 = Packed32<Wrapping<usize>, U16>;

    pub type isizex2 = Packed8<isize, U2>;
    pub type isizex4 = Packed16<isize, U4>;
    pub type isizex8 = Packed32<isize, U8>;
    pub type isizex16 = Packed32<isize, U16>;

    pub type wisizex2 = Packed8<Wrapping<isize>, U2>;
    pub type wisizex4 = Packed16<Wrapping<isize>, U4>;
    pub type wisizex8 = Packed32<Wrapping<isize>, U8>;
    pub type wisizex16 = Packed32<Wrapping<isize>, U16>;
}

#[cfg(target_pointer_width = "64")]
mod sized {
    use super::*;

    pub type msizex2 = Packed16<msize, U2>;
    pub type msizex4 = Packed32<msize, U4>;
    pub type msizex8 = Packed32<msize, U8>;
    pub type msizex16 = Packed32<msize, U16>;

    pub type usizex2 = Packed16<usize, U2>;
    pub type usizex4 = Packed32<usize, U4>;
    pub type usizex8 = Packed32<usize, U8>;
    pub type usizex16 = Packed32<usize, U16>;

    pub type wusizex2 = Packed16<Wrapping<usize>, U2>;
    pub type wusizex4 = Packed32<Wrapping<usize>, U4>;
    pub type wusizex8 = Packed32<Wrapping<usize>, U8>;
    pub type wusizex16 = Packed32<Wrapping<usize>, U16>;

    pub type isizex2 = Packed16<isize, U2>;
    pub type isizex4 = Packed32<isize, U4>;
    pub type isizex8 = Packed32<isize, U8>;
    pub type isizex16 = Packed32<isize, U16>;

    pub type wisizex2 = Packed16<Wrapping<isize>, U2>;
    pub type wisizex4 = Packed32<Wrapping<isize>, U4>;
    pub type wisizex8 = Packed32<Wrapping<isize>, U8>;
    pub type wisizex16 = Packed32<Wrapping<isize>, U16>;
}

pub use sized::*;
