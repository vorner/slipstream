use core::num::Wrapping;

use typenum::consts::*;

use crate::vector::*;

pub type u16x4 = Packed8<u16, Wrapping<u16>, U4>;
pub type u16x8 = Packed16<u16, Wrapping<u16>, U8>;
pub type u32x4 = Packed16<u32, Wrapping<u32>, U4>;
pub type u32x16 = Packed64<u32, Wrapping<u32>, U16>;
