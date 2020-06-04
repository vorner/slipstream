#![allow(non_camel_case_types)]
#![cfg_attr(not(test), no_std)]

use generic_array::ArrayLength;
use typenum::marker_traits::Unsigned;

mod iterators;
pub mod mask;
pub mod vector;
pub mod types;

pub use iterators::Vectorizable;
pub use mask::Mask;
pub use types::*;

pub mod prelude {
    pub use crate::Mask as _;
    pub use crate::Vector as _;
    pub use crate::Vectorizable as _;
    pub use crate::types::*;
}

mod inner {
    use core::num::Wrapping;

    use crate::mask::{m8, m16, m32, m64, m128, msize, Mask};

    pub unsafe trait Repr: Send + Sync + Copy + 'static {
        type Mask: Mask;
        const ONE: Self;
    }

    unsafe impl Repr for Wrapping<u8> {
        type Mask = m8;
        const ONE: Wrapping<u8> = Wrapping(1);
    }
    unsafe impl Repr for Wrapping<u16> {
        type Mask = m16;
        const ONE: Wrapping<u16> = Wrapping(1);
    }
    unsafe impl Repr for Wrapping<u32> {
        type Mask = m32;
        const ONE: Wrapping<u32> = Wrapping(1);
    }
    unsafe impl Repr for Wrapping<u64> {
        type Mask = m64;
        const ONE: Wrapping<u64> = Wrapping(1);
    }
    unsafe impl Repr for Wrapping<u128> {
        type Mask = m128;
        const ONE: Wrapping<u128> = Wrapping(1);
    }
    unsafe impl Repr for Wrapping<usize> {
        type Mask = msize;
        const ONE: Wrapping<usize> = Wrapping(1);
    }
    unsafe impl Repr for u8 {
        type Mask = m8;
        const ONE: u8 = 1;
    }
    unsafe impl Repr for u16 {
        type Mask = m16;
        const ONE: u16 = 1;
    }
    unsafe impl Repr for u32 {
        type Mask = m32;
        const ONE: u32 = 1;
    }
    unsafe impl Repr for u64 {
        type Mask = m64;
        const ONE: u64 = 1;
    }
    unsafe impl Repr for u128 {
        type Mask = m128;
        const ONE: u128 = 1;
    }
    unsafe impl Repr for usize {
        type Mask = msize;
        const ONE: usize = 1;
    }

    unsafe impl Repr for Wrapping<i8> {
        type Mask = m8;
        const ONE: Wrapping<i8> = Wrapping(1);
    }
    unsafe impl Repr for Wrapping<i16> {
        type Mask = m16;
        const ONE: Wrapping<i16> = Wrapping(1);
    }
    unsafe impl Repr for Wrapping<i32> {
        type Mask = m32;
        const ONE: Wrapping<i32> = Wrapping(1);
    }
    unsafe impl Repr for Wrapping<i64> {
        type Mask = m64;
        const ONE: Wrapping<i64> = Wrapping(1);
    }
    unsafe impl Repr for Wrapping<i128> {
        type Mask = m128;
        const ONE: Wrapping<i128> = Wrapping(1);
    }
    unsafe impl Repr for Wrapping<isize> {
        type Mask = msize;
        const ONE: Wrapping<isize> = Wrapping(1);
    }
    unsafe impl Repr for i8 {
        type Mask = m8;
        const ONE: i8 = 1;
    }
    unsafe impl Repr for i16 {
        type Mask = m16;
        const ONE: i16 = 1;
    }
    unsafe impl Repr for i32 {
        type Mask = m32;
        const ONE: i32 = 1;
    }
    unsafe impl Repr for i64 {
        type Mask = m64;
        const ONE: i64 = 1;
    }
    unsafe impl Repr for i128 {
        type Mask = m128;
        const ONE: i128 = 1;
    }
    unsafe impl Repr for isize {
        type Mask = msize;
        const ONE: isize = 1;
    }

    unsafe impl Repr for f32 {
        type Mask = m32;
        const ONE: f32 = 1.0;
    }
    unsafe impl Repr for f64 {
        type Mask = m64;
        const ONE: f64 = 1.0;
    }
    unsafe impl<M: Mask> Repr for M {
        type Mask = Self;
        const ONE: M = M::TRUE;
    }
}

pub trait Vector: Copy + Send + Sync + Sized + 'static {
    type Base: inner::Repr;
    type Lanes: ArrayLength<Self::Base>;
    type Mask;
    const LANES: usize = Self::Lanes::USIZE;
    unsafe fn new_unchecked(input: *const Self::Base) -> Self;

    #[inline]
    fn new<I>(input: I) -> Self
    where
        I: AsRef<[Self::Base]>,
    {
        let input = input.as_ref();
        assert_eq!(
            input.len(),
            Self::LANES,
            "Creating vector from the wrong sized slice (expected {}, got {})",
            Self::LANES, input.len(),
        );
        unsafe { Self::new_unchecked(input.as_ptr()) }
    }

    fn splat(value: Self::Base) -> Self;

    fn gather_load<I, Idx>(input: I, idx: Idx) -> Self
    where
        I: AsRef<[Self::Base]>,
        Idx: AsRef<[usize]>;

    fn scatter_store<O, Idx>(self, output: O, idx: Idx)
    where
        O: AsMut<[Self::Base]>,
        Idx: AsRef<[usize]>;

    fn horizontal_sum(self) -> Self::Base;
    fn horizontal_product(self) -> Self::Base;
}
