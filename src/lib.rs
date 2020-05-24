#![cfg_attr(not(feature = "std"), no_std)]
#![allow(non_camel_case_types)]

use core::fmt::{Debug, Display, Formatter, Result as FmtResult};
use core::iter;
use core::marker::PhantomData;
use core::mem;
use core::ops::*;

use generic_array::{ArrayLength, GenericArray};
use typenum::consts::*;
use typenum::marker_traits::Unsigned;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub mod avx;
pub mod polyfill;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub mod sse;
mod vector;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub use avx::Avx2;
pub use polyfill::Polyfill;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub use sse::Sse4_1;

mod inner {
    use core::num::Wrapping;

    use typenum::consts::*;
    use typenum::marker_traits::Unsigned;

    pub trait InstructionSet: Sized { }

    pub unsafe trait Repr<For>: Copy {
        // XXX Rename
        type LaneMultiplyier: Unsigned;
    }

    unsafe impl Repr<u16> for Wrapping<u16> {
        type LaneMultiplyier = U1;
    }

    unsafe impl Repr<u32> for Wrapping<u32> {
        type LaneMultiplyier = U1;
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct InstructionSetNotAvailable(pub &'static str);

impl Display for InstructionSetNotAvailable {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        write!(fmt, "Instruction set {} not available", self.0)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for InstructionSetNotAvailable {}

pub trait InstructionSet: Copy + Debug + inner::InstructionSet {
    #[inline]
    fn load<V, B>(self, input: &[B]) -> V
    where
        V: Vector<B, Self>
    {
        V::new(input, self)
    }

    #[inline]
    fn splat<V, B>(self, value: B) -> V
    where
        V: Vector<B, Self>,
        B: Copy,
    {
        V::splat(value, self)
    }

    // XXX Better name? Like, split?
    #[inline]
    fn iter<B, V>(self, arr: &[B]) -> Iter<V, B, Self>
    where
        V: Vector<B, Self>,
    {
        Iter {
            data: arr,
            ins: self,
            _v: PhantomData,
        }
    }

    #[inline]
    fn iter_mut<B, V>(self, arr: &mut [B]) -> IterMut<V, B, Self>
    where
        V: Vector<B, Self>,
    {
        IterMut {
            data: arr,
            ins: self,
            _v: PhantomData,
        }
    }

    fn detect() -> Result<Self, InstructionSetNotAvailable>;
    type u16x4: IntVector<u16, Self, Lanes = U4>;
    type u16x8: IntVector<u16, Self, Lanes = U8>;
    type u32x16: IntVector<u32, Self, Lanes = U16>;

    type u16s: IntVector<u16, Self>;
    type u32s: IntVector<u32, Self>;
}

#[derive(Copy, Clone, Debug)]
pub struct Iter<'a, V, B, I> {
    data: &'a [B],
    ins: I,
    _v: PhantomData<V>,
}

impl<V, B, I> Iterator for Iter<'_, V, B, I>
where
    V: Vector<B, I>,
    B: Default + Copy,
    I: Copy,
{
    type Item = V;
    #[inline]
    fn next(&mut self) -> Option<V> {
        if self.data.len() >= V::LANES {
            let (start, rest) = self.data.split_at(V::LANES);
            self.data = rest;
            Some(V::new(start, self.ins))
        } else if self.data.is_empty() {
            None
        } else {
            let mut vec = V::splat(Default::default(), self.ins);
            vec.deref_mut()[..self.data.len()].copy_from_slice(self.data);
            self.data = &[];
            Some(vec)
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = (self.data.len() + V::LANES - 1) / V::LANES;
        (len, Some(len))
    }
}

pub struct IterMut<'a, V, B, I> {
    data: &'a mut [B],
    ins: I,
    _v: PhantomData<V>,
}

impl<'a, V, B, I> Iterator for IterMut<'a, V, B, I>
where
    V: Vector<B, I>,
    B: Default + Copy,
    I: Copy,
{
    type Item = MutProxy<'a, V, B>;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.data.len() >= V::LANES {
            let data = mem::take(&mut self.data);
            let (start, rest) = data.split_at_mut(V::LANES);
            self.data = rest;
            let data = V::new(start, self.ins);
            Some(MutProxy {
                data,
                restore: start,
            })
        } else if self.data.is_empty() {
            None
        } else {
            let mut data = V::splat(Default::default(), self.ins);
            let restore = mem::take(&mut self.data);
            data.deref_mut()[..self.data.len()].copy_from_slice(self.data);
            Some(MutProxy {
                data,
                restore,
            })
        }
    }
}

#[derive(Debug)]
pub struct MutProxy<'a, V, B>
where
    V: Deref<Target = [B]>,
    B: Copy,
{
    data: V,
    restore: &'a mut [B],
}

impl<V, B> Deref for MutProxy<'_, V, B>
where
    V: Deref<Target = [B]>,
    B: Copy,
{
    type Target = V;
    fn deref(&self) -> &V {
        &self.data
    }
}

impl<V, B> DerefMut for MutProxy<'_, V, B>
where
    V: Deref<Target = [B]>,
    B: Copy,
{
    fn deref_mut(&mut self) -> &mut V {
        &mut self.data
    }
}

impl<V, B> Drop for MutProxy<'_, V, B>
where
    V: Deref<Target = [B]>,
    B: Copy,
{
    #[inline]
    fn drop(&mut self) {
        self.restore.copy_from_slice(&self.data.deref()[..self.restore.len()]);
    }
}

pub trait Vector<B, I>:
    Deref<Target = [B]> + DerefMut +
    Sized
{
    type Lanes: ArrayLength<B>;
    const LANES: usize = Self::Lanes::USIZE;
    // TODO: new_unchecked ‒ aligned, no instruction set checked
    fn new(input: &[B], instruction_set: I) -> Self;

    #[inline]
    fn splat(value: B, instruction_set: I) -> Self
    where
        B: Copy,
    {
        let input = iter::repeat(value)
            .take(Self::LANES)
            .collect::<GenericArray<B, Self::Lanes>>();
        Self::new(&input, instruction_set)
    }
}

pub trait IntVector<B, I>:
    Copy + Send + Sync + 'static +
    Vector<B, I> +
    Add<Output = Self> + AddAssign + Sub<Output = Self> + SubAssign +
    Mul<Output = Self> + MulAssign
{
}

impl<V, B, I> IntVector<B, I> for V
where
    V: Copy + Send + Sync + 'static +
        Vector<B, I> +
        Add<Output = Self> + AddAssign + Sub<Output = Self> + SubAssign +
        Mul<Output = Self> + MulAssign
{}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iter() {
        fn inner<I: InstructionSet>() {
            let data = (0..=10u16).collect::<Vec<_>>();
            let is = I::detect().unwrap();
            let mut acc: I::u16s = is.splat(0);
            for v in is.iter(&data) {
                acc += v;
            }
            let total: u16 = acc.iter().sum();
            assert_eq!(total, 55);
        }
        inner::<Polyfill>();
        inner::<Sse4_1>();
    }

    #[test]
    fn iter_poly() {
        let data = (0..=10u16).collect::<Vec<_>>();
        let is = Polyfill;
        let vtotal = is.iter(&data)
            .fold(polyfill::u16s::splat(0u16, is), |a, b| a + b);
        let total: u16 = vtotal.iter().sum();
        assert_eq!(total, 55);
    }

    #[test]
    fn iter_mut() {
        fn inner<I: InstructionSet>() {
            let data = (0..33u32).collect::<Vec<_>>();
            let mut dst = [0u32; 33];
            let is = I::detect().unwrap();
            let ones: I::u32s = is.splat(1);
            for (mut d, s) in is.iter_mut(&mut dst).zip(is.iter(&data)) {
                *d = ones + s;
            }

            for (l, r) in data.iter().zip(dst.iter()) {
                assert_eq!(*l + 1, *r);
            }
        }
        inner::<Polyfill>();
        inner::<Sse4_1>();
    }
}
