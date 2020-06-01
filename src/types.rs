use core::num::Wrapping;

use typenum::consts::*;

use crate::vector::*;

pub type u8x2 = Packed2<u8, u8, U2>;
pub type u8x4 = Packed4<u8, u8, U4>;
pub type u8x8 = Packed8<u8, u8, U8>;
pub type u8x16 = Packed16<u8, u8, U16>;

pub type u16x2 = Packed4<u16, u16, U2>;
pub type u16x4 = Packed8<u16, u16, U4>;
pub type u16x8 = Packed16<u16, u16, U8>;
pub type u16x16 = Packed32<u16, u16, U16>;

pub type u32x2 = Packed8<u32, u32, U2>;
pub type u32x4 = Packed16<u32, u32, U4>;
pub type u32x8 = Packed32<u32, u32, U8>;
pub type u32x16 = Packed32<u32, u32, U16>;

pub type u64x2 = Packed16<u64, u64, U2>;
pub type u64x4 = Packed32<u64, u64, U4>;
pub type u64x8 = Packed32<u64, u64, U8>;
pub type u64x16 = Packed32<u64, u64, U16>;

pub type wu8x2 = Packed2<u8, Wrapping<u8>, U2>;
pub type wu8x4 = Packed4<u8, Wrapping<u8>, U4>;
pub type wu8x8 = Packed8<u8, Wrapping<u8>, U8>;
pub type wu8x16 = Packed16<u8, Wrapping<u8>, U16>;

pub type wu16x2 = Packed4<u16, Wrapping<u16>, U2>;
pub type wu16x4 = Packed8<u16, Wrapping<u16>, U4>;
pub type wu16x8 = Packed16<u16, Wrapping<u16>, U8>;
pub type wu16x16 = Packed32<u16, Wrapping<u16>, U16>;

pub type wu32x2 = Packed8<u32, Wrapping<u32>, U2>;
pub type wu32x4 = Packed16<u32, Wrapping<u32>, U4>;
pub type wu32x8 = Packed32<u32, Wrapping<u32>, U8>;
pub type wu32x16 = Packed32<u32, Wrapping<u32>, U16>;

pub type wu64x2 = Packed16<u64, Wrapping<u64>, U2>;
pub type wu64x4 = Packed32<u64, Wrapping<u64>, U4>;
pub type wu64x8 = Packed32<u64, Wrapping<u64>, U8>;
pub type wu64x16 = Packed32<u64, Wrapping<u64>, U16>;

// TODO: u/i not wrapping
// TODO: usize vectors ‒ alignments based on arch

pub type i8x2 = Packed2<i8, i8, U2>;
pub type i8x4 = Packed4<i8, i8, U4>;
pub type i8x8 = Packed8<i8, i8, U8>;
pub type i8x16 = Packed16<i8, i8, U16>;

pub type i16x2 = Packed4<i16, i16, U2>;
pub type i16x4 = Packed8<i16, i16, U4>;
pub type i16x8 = Packed16<i16, i16, U8>;
pub type i16x16 = Packed32<i16, i16, U16>;

pub type i32x2 = Packed8<i32, i32, U2>;
pub type i32x4 = Packed16<i32, i32, U4>;
pub type i32x8 = Packed32<i32, i32, U8>;
pub type i32x16 = Packed32<i32, i32, U16>;

pub type i64x2 = Packed16<i64, i64, U2>;
pub type i64x4 = Packed32<i64, i64, U4>;
pub type i64x8 = Packed32<i64, i64, U8>;
pub type i64x16 = Packed32<i64, i64, U16>;

pub type wi8x2 = Packed2<i8, Wrapping<i8>, U2>;
pub type wi8x4 = Packed4<i8, Wrapping<i8>, U4>;
pub type wi8x8 = Packed8<i8, Wrapping<i8>, U8>;
pub type wi8x16 = Packed16<i8, Wrapping<i8>, U16>;

pub type wi16x2 = Packed4<i16, Wrapping<i16>, U2>;
pub type wi16x4 = Packed8<i16, Wrapping<i16>, U4>;
pub type wi16x8 = Packed16<i16, Wrapping<i16>, U8>;
pub type wi16x16 = Packed32<i16, Wrapping<i16>, U16>;

pub type wi32x2 = Packed8<i32, Wrapping<i32>, U2>;
pub type wi32x4 = Packed16<i32, Wrapping<i32>, U4>;
pub type wi32x8 = Packed32<i32, Wrapping<i32>, U8>;
pub type wi32x16 = Packed32<i32, Wrapping<i32>, U16>;

pub type wi64x2 = Packed16<i64, Wrapping<i64>, U2>;
pub type wi64x4 = Packed32<i64, Wrapping<i64>, U4>;
pub type wi64x8 = Packed32<i64, Wrapping<i64>, U8>;
pub type wi64x16 = Packed32<i64, Wrapping<i64>, U16>;

pub type f32x2 = Packed8<f32, f32, U2>;
pub type f32x4 = Packed16<f32, f32, U4>;
pub type f32x8 = Packed32<f32, f32, U8>;
pub type f32x16 = Packed32<f32, f32, U16>;

pub type f64x2 = Packed16<f64, f64, U2>;
pub type f64x4 = Packed32<f64, f64, U4>;
pub type f64x8 = Packed32<f64, f64, U8>;
pub type f64x16 = Packed32<f64, f64, U16>;
