use core::num::Wrapping;

use typenum::consts::*;

use crate::{InstructionSet, InstructionSetNotAvailable};
use crate::vector::VectorImpl;

// It's OK to let users create this one directly, it's safe to use always.
#[derive(Copy, Clone, Debug)]
pub struct Polyfill;

impl crate::inner::InstructionSet for Polyfill { }

impl InstructionSet for Polyfill {
    #[inline]
    fn detect() -> Result<Self, InstructionSetNotAvailable> {
        Ok(Self)
    }
    type u16x8 = u16x8;
    type u16x4 = u16x4;
    type u32x16 = u32x16;

    type u16s = u16s;
    type u32s = u32s;
}

pub type u16x8 = VectorImpl<u16, Wrapping<u16>, U8, Polyfill>;
pub type u16x4 = VectorImpl<u16, Wrapping<u16>, U4, Polyfill>;
pub type u32x16 = VectorImpl<u32, Wrapping<u32>, U16, Polyfill>;

pub type u16s = VectorImpl<u16, Wrapping<u16>, U1, Polyfill>;
pub type u32s = VectorImpl<u32, Wrapping<u32>, U1, Polyfill>;
