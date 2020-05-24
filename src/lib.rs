#![cfg_attr(not(feature = "std"), no_std)]
#![allow(non_camel_case_types)]

use core::fmt::{Debug, Display, Formatter, Result as FmtResult};
use core::iter;
use core::marker::PhantomData;
use core::mem;
use core::num::Wrapping;
use core::ops::*;
use core::ptr;
use core::slice;

use generic_array::{ArrayLength, GenericArray};
use typenum::consts::*;
use typenum::marker_traits::Unsigned;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod avx;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod sse;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub use avx::Avx2;
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
    fn load_u16x4(self, input: &[u16]) -> Self::u16x4 {
        self.load(input)
    }

    #[inline]
    fn load_u16x8(self, input: &[u16]) -> Self::u16x8 {
        self.load(input)
    }

    #[inline]
    fn load_u32x16(self, input: &[u32]) -> Self::u32x16 {
        self.load(input)
    }

    #[inline]
    fn load_u16s(self, input: &[u16]) -> Self::u16s {
        self.load(input)
    }

    #[inline]
    fn load_u32s(self, input: &[u32]) -> Self::u32s {
        self.load(input)
    }

    #[inline]
    fn splat<V, B>(self, value: B) -> V
    where
        V: Vector<B, Self>,
        B: Copy,
    {
        V::splat(value, self)
    }

    #[inline]
    fn splat_u16x4(self, input: u16) -> Self::u16x4 {
        self.splat(input)
    }

    #[inline]
    fn splat_u16x8(self, input: u16) -> Self::u16x8 {
        self.splat(input)
    }

    #[inline]
    fn splat_u32x16(self, input: u32) -> Self::u32x16 {
        self.splat(input)
    }

    #[inline]
    fn splat_u16s(self, input: u16) -> Self::u16s {
        self.splat(input)
    }

    #[inline]
    fn splat_u32s(self, input: u32) -> Self::u32s {
        self.splat(input)
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

// It's OK to let users create this one directly, it's safe to use always.
#[derive(Copy, Clone, Debug)]
pub struct Polyfill;

impl inner::InstructionSet for Polyfill { }

impl InstructionSet for Polyfill {
    #[inline]
    fn detect() -> Result<Self, InstructionSetNotAvailable> {
        Ok(Self)
    }
    type u16x8 = VectorImpl<u16, Wrapping<u16>, U8, Polyfill>;
    type u16x4 = VectorImpl<u16, Wrapping<u16>, U4, Polyfill>;
    type u32x16 = VectorImpl<u32, Wrapping<u32>, U16, Polyfill>;

    type u16s = VectorImpl<u16, Wrapping<u16>, U1, Polyfill>;
    type u32s = VectorImpl<u32, Wrapping<u32>, U1, Polyfill>;
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

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct VectorImpl<B, R, S, I>
where
    R: inner::Repr<B>,
    S: Unsigned + Mul<R::LaneMultiplyier> + ArrayLength<R>,
    S::ArrayType: Copy,
{
    content: GenericArray<R, S>,
    _props: PhantomData<(B, I, <S as Mul<R::LaneMultiplyier>>::Output)>,
}

impl<B, R, S, I> Vector<B, I> for VectorImpl<B, R, S, I>
where
    R: inner::Repr<B>,
    S: Unsigned + Mul<R::LaneMultiplyier> + ArrayLength<R>,
    S::ArrayType: Copy,
    <S as Mul<R::LaneMultiplyier>>::Output: ArrayLength<B>,
{
    type Lanes = <S as Mul<R::LaneMultiplyier>>::Output;
    #[inline]
    fn new(input: &[B], _instruction_set: I) -> Self {
        assert_eq!(
            input.len(),
            S::USIZE * R::LaneMultiplyier::USIZE,
            "Creating vector from the wrong sized slice",
        );
        unsafe {
            let content = ptr::read_unaligned(input.as_ptr().cast());
            Self {
                content,
                _props: PhantomData,
            }
        }
    }

}

impl<B, R, S, I> Deref for VectorImpl<B, R, S, I>
where
    R: inner::Repr<B>,
    S: Unsigned + Mul<R::LaneMultiplyier> + ArrayLength<R>,
    S::ArrayType: Copy,
{
    type Target = [B];
    #[inline]
    fn deref(&self) -> &[B] {
        unsafe {
            slice::from_raw_parts(
                self.content.as_ptr().cast(),
                S::USIZE * R::LaneMultiplyier::USIZE,
            )
        }
    }
}

impl<B, R, S, I> DerefMut for VectorImpl<B, R, S, I>
where
    R: inner::Repr<B>,
    S: Unsigned + Mul<R::LaneMultiplyier> + ArrayLength<R>,
    S::ArrayType: Copy,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut [B] {
        unsafe {
            slice::from_raw_parts_mut(
                self.content.as_mut_ptr().cast(),
                S::USIZE * R::LaneMultiplyier::USIZE,
            )
        }
    }
}

impl<B, R, S, I> Index<usize> for VectorImpl<B, R, S, I>
where
    R: inner::Repr<B>,
    S: Unsigned + Mul<R::LaneMultiplyier> + ArrayLength<R>,
    S::ArrayType: Copy,
{
    type Output = B;
    #[inline]
    fn index(&self, idx: usize) -> &B {
        self.deref().index(idx)
    }
}

impl<B, R, S, I> IndexMut<usize> for VectorImpl<B, R, S, I>
where
    R: inner::Repr<B>,
    S: Unsigned + Mul<R::LaneMultiplyier> + ArrayLength<R>,
    S::ArrayType: Copy,
{
    #[inline]
    fn index_mut(&mut self, idx: usize) -> &mut B {
        self.deref_mut().index_mut(idx)
    }
}

macro_rules! bin_op_impl {
    ($tr: ident, $meth: ident, $tr_assign: ident, $meth_assign: ident) => {
        impl<B, R, S, I> $tr for VectorImpl<B, R, S, I>
        where
            R: inner::Repr<B> + $tr<Output = R> + Copy,
            S: Unsigned + Mul<R::LaneMultiplyier> + ArrayLength<R>,
            S::ArrayType: Copy,
        {
            type Output = Self;
            #[inline]
            fn $meth(self, rhs: Self) -> Self {
                let content = self.content.iter()
                    .zip(rhs.content.iter())
                    .map(|(a, b)| $tr::$meth(*a, *b))
                    .collect();
                Self {
                    content,
                    _props: PhantomData,
                }
            }
        }

        impl<B, R, S, I> $tr_assign for VectorImpl<B, R, S, I>
        where
            R: inner::Repr<B> + $tr_assign + Copy,
            S: Unsigned + Mul<R::LaneMultiplyier> + ArrayLength<R>,
            S::ArrayType: Copy,
        {
            #[inline]
            fn $meth_assign(&mut self, rhs: Self) {
                for (r, s) in self.content.iter_mut().zip(rhs.content.iter()) {
                    $tr_assign::$meth_assign(r, *s);
                }
            }
        }
    }
}

bin_op_impl!(Add, add, AddAssign, add_assign);
bin_op_impl!(Sub, sub, SubAssign, sub_assign);
bin_op_impl!(Mul, mul, MulAssign, mul_assign);
bin_op_impl!(Div, div, DivAssign, div_assign);
bin_op_impl!(Rem, rem, RemAssign, rem_assign);
bin_op_impl!(BitAnd, bitand, BitAndAssign, bitand_assign);
bin_op_impl!(BitOr, bitor, BitOrAssign, bitor_assign);
bin_op_impl!(BitXor, bitxor, BitXorAssign, bitxor_assign);

macro_rules! una_op_impl {
    ($tr: ident, $meth: ident) => {
        impl<B, R, S, I> $tr for VectorImpl<B, R, S, I>
        where
            R: inner::Repr<B> + $tr<Output = R> + Copy,
            S: Unsigned + Mul<R::LaneMultiplyier> + ArrayLength<R>,
            S::ArrayType: Copy,
        {
            type Output = Self;
            fn $meth(self) -> Self {
                let content = self.content
                    .iter()
                    .copied()
                    .map($tr::$meth)
                    .collect();
                Self {
                    content,
                    _props: PhantomData,
                }
            }
        }
    }
}

una_op_impl!(Neg, neg);
una_op_impl!(Not, not);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iter() {
        fn inner<I: InstructionSet>() {
            let data = (0..=10u16).collect::<Vec<_>>();
            let is = I::detect().unwrap();
            let mut acc = is.splat_u16s(0);
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
            .fold(is.splat_u16s(0u16), |a, b| a + b);
        let total: u16 = vtotal.iter().sum();
        assert_eq!(total, 55);
    }

    #[test]
    fn iter_mut() {
        fn inner<I: InstructionSet>() {
            let data = (0..33u32).collect::<Vec<_>>();
            let mut dst = [0u32; 33];
            let is = I::detect().unwrap();
            let ones = is.splat_u32s(1);
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
