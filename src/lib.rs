#![allow(non_camel_case_types)]
#![cfg_attr(not(test), no_std)]

use generic_array::ArrayLength;
use typenum::marker_traits::Unsigned;

mod iterators;
pub mod vector;
pub mod types;

pub use iterators::Vectorizable;
pub use types::*;

pub mod prelude {
    pub use crate::Vector;
    pub use crate::Vectorizable;
    pub use crate::types::*;
}

mod inner {
    use core::num::Wrapping;

    pub unsafe trait Repr: Send + Sync + Copy + 'static {
        const ONE: Self;
    }

    unsafe impl Repr for Wrapping<u8> {
        const ONE: Wrapping<u8> = Wrapping(1);
    }
    unsafe impl Repr for Wrapping<u16> {
        const ONE: Wrapping<u16> = Wrapping(1);
    }
    unsafe impl Repr for Wrapping<u32> {
        const ONE: Wrapping<u32> = Wrapping(1);
    }
    unsafe impl Repr for Wrapping<u64> {
        const ONE: Wrapping<u64> = Wrapping(1);
    }
    unsafe impl Repr for Wrapping<u128> {
        const ONE: Wrapping<u128> = Wrapping(1);
    }
    unsafe impl Repr for Wrapping<usize> {
        const ONE: Wrapping<usize> = Wrapping(1);
    }
    unsafe impl Repr for u8 {
        const ONE: u8 = 1;
    }
    unsafe impl Repr for u16 {
        const ONE: u16 = 1;
    }
    unsafe impl Repr for u32 {
        const ONE: u32 = 1;
    }
    unsafe impl Repr for u64 {
        const ONE: u64 = 1;
    }
    unsafe impl Repr for u128 {
        const ONE: u128 = 1;
    }
    unsafe impl Repr for usize {
        const ONE: usize = 1;
    }

    unsafe impl Repr for Wrapping<i8> {
        const ONE: Wrapping<i8> = Wrapping(1);
    }
    unsafe impl Repr for Wrapping<i16> {
        const ONE: Wrapping<i16> = Wrapping(1);
    }
    unsafe impl Repr for Wrapping<i32> {
        const ONE: Wrapping<i32> = Wrapping(1);
    }
    unsafe impl Repr for Wrapping<i64> {
        const ONE: Wrapping<i64> = Wrapping(1);
    }
    unsafe impl Repr for Wrapping<i128> {
        const ONE: Wrapping<i128> = Wrapping(1);
    }
    unsafe impl Repr for Wrapping<isize> {
        const ONE: Wrapping<isize> = Wrapping(1);
    }
    unsafe impl Repr for i8 {
        const ONE: i8 = 1;
    }
    unsafe impl Repr for i16 {
        const ONE: i16 = 1;
    }
    unsafe impl Repr for i32 {
        const ONE: i32 = 1;
    }
    unsafe impl Repr for i64 {
        const ONE: i64 = 1;
    }
    unsafe impl Repr for i128 {
        const ONE: i128 = 1;
    }
    unsafe impl Repr for isize {
        const ONE: isize = 1;
    }

    unsafe impl Repr for f32 {
        const ONE: f32 = 1.0;
    }
    unsafe impl Repr for f64 {
        const ONE: f64 = 1.0;
    }
}

pub trait Vector: Copy + Send + Sync + Sized + 'static {
    type Base: inner::Repr;
    type Lanes: ArrayLength<Self::Base>;
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
