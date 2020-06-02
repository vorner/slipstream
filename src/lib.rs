#![allow(non_camel_case_types)]
#![cfg_attr(not(test), no_std)]

use core::marker::PhantomData;
use core::mem;
use core::ops::*;
use core::slice;

use generic_array::ArrayLength;
use typenum::marker_traits::Unsigned;

pub mod vector;
pub mod types;

pub use types::*;

pub mod prelude {
    pub use crate::Vector;
    pub use crate::Vectorizable;
    pub use crate::types::*;
}

mod inner {
    use core::num::Wrapping;

    pub unsafe trait Repr: Copy {
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

#[derive(Debug)]
pub struct MutProxy<'a, B, V>
where
    V: Deref<Target = [B]>,
    B: Copy,
{
    data: V,
    restore: &'a mut [B],
}

impl<B, V> Deref for MutProxy<'_, B, V>
where
    V: Deref<Target = [B]>,
    B: Copy,
{
    type Target = V;
    #[inline]
    fn deref(&self) -> &V {
        &self.data
    }
}

impl<B, V> DerefMut for MutProxy<'_, B, V>
where
    V: Deref<Target = [B]>,
    B: Copy,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut V {
        &mut self.data
    }
}

impl<B, V> Drop for MutProxy<'_, B, V>
where
    V: Deref<Target = [B]>,
    B: Copy,
{
    #[inline]
    fn drop(&mut self) {
        self.restore.copy_from_slice(&self.data.deref()[..self.restore.len()]);
    }
}

pub trait Vector<B>: Deref<Target = [B]> + DerefMut + Sized + 'static {
    type Lanes: ArrayLength<B>;
    const LANES: usize = Self::Lanes::USIZE;
    unsafe fn new_unchecked(input: *const B) -> Self;

    #[inline]
    fn new(input: &[B]) -> Self {
        assert_eq!(
            input.len(),
            Self::LANES,
            "Creating vector from the wrong sized slice (expected {}, got {})",
            Self::LANES, input.len(),
        );
        unsafe { Self::new_unchecked(input.as_ptr()) }
    }

    fn splat(value: B) -> Self
    where
        B: Copy;

    fn horizontal_sum(self) -> B;
    fn horizontal_product(self) -> B;
}

// TODO: Hide away inside inner
pub trait Partial<V> {
    fn take_partial(&mut self) -> Option<V>;
    fn size(&self) -> usize;
}

impl<V> Partial<V> for () {
    #[inline]
    fn take_partial(&mut self) -> Option<V> {
        None
    }
    #[inline]
    fn size(&self) -> usize {
        0
    }
}

impl<V> Partial<V> for Option<V> {
    #[inline]
    fn take_partial(&mut self) -> Option<V> {
        Option::take(self)
    }
    fn size(&self) -> usize {
        self.is_some() as usize
    }
}
// TODO: Hide away
pub trait Vectorizer<R> {
    // Safety:
    // idx in range
    // will be called at most once for each idx
    unsafe fn get(&self, idx: usize) -> R;
}

#[derive(Copy, Clone, Debug)]
pub struct VectorizedIter<V, P, R> {
    partial: P,
    vectorizer: V,
    left: usize,
    right: usize,
    _result: PhantomData<R>,
}

impl<V, P, R> Iterator for VectorizedIter<V, P, R>
where
    V: Vectorizer<R>,
    P: Partial<R>,
{
    type Item = R;

    #[inline]
    fn next(&mut self) -> Option<R> {
        if self.left < self.right {
            let idx = self.left;
            self.left += 1;
            Some(unsafe { self.vectorizer.get(idx) })
        } else if let Some(partial) = self.partial.take_partial() {
            Some(partial)
        } else {
            None
        }
    }
}

// ExactSizedIterator, ReverseIterator

// TODO: Hide away the basic implementation?
// TODO: Is it a good idea to have it like vec.vectorize()? Won't it create footguns on mut vector?
pub trait Vectorizable<V>: Sized {
    type Padding;
    type Vectorizer: Vectorizer<V>;
    fn create(self, pad: Option<Self::Padding>) -> (Self::Vectorizer, usize, Option<V>);

    #[inline]
    fn vectorize(self) -> VectorizedIter<Self::Vectorizer, (), V> {
        let (vectorizer, len, partial) = self.create(None);
        assert!(partial.is_none());
        VectorizedIter {
            partial: (),
            vectorizer,
            left: 0,
            right: len,
            _result: PhantomData,
        }
    }

    #[inline]
    fn vectorize_pad(self, pad: Self::Padding) -> VectorizedIter<Self::Vectorizer, Option<V>, V> {
        let (vectorizer, len, partial) = self.create(Some(pad));
        VectorizedIter {
            partial,
            vectorizer,
            left: 0,
            right: len,
            _result: PhantomData,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ReadVectorizer<'a, B, V> {
    start: *const B,
    _vector: PhantomData<V>,
    _slice: PhantomData<&'a [B]>, // To hold the lifetime
}

// Note: The impls here assume V, B, P are Sync and Send, which they are. Nobody is able to create
// this directly and we do have the limits on Vector, the allowed implementations, etc.
unsafe impl<B, V> Send for ReadVectorizer<'_, B, V> {}
unsafe impl<B, V> Sync for ReadVectorizer<'_, B, V> {}

impl<'a, B, V> Vectorizer<V> for ReadVectorizer<'_, B, V>
where
    V: Vector<B>,
    B: Copy,
{
    #[inline]
    unsafe fn get(&self, idx: usize) -> V {
        V::new_unchecked(self.start.add(V::LANES * idx))
    }
}

impl<'a, B, V> Vectorizable<V> for &'a [B]
where
    B: Copy + 'a,
    V: Vector<B>,
{
    type Vectorizer = ReadVectorizer<'a, B, V>;
    type Padding = V;
    #[inline]
    fn create(self, pad: Option<V>) -> (Self::Vectorizer, usize, Option<V>) {
        let len = self.len();
        assert!(len * mem::size_of::<B>() <= isize::MAX as usize, "Slice too huge");
        let rest = len % V::LANES;
        let main = len - rest;
        let start = self.as_ptr();
        let partial = match (rest, pad) {
            (0, _) => None,
            (_, Some(mut pad)) => {
                pad[..rest].copy_from_slice(&self[main..]);
                Some(pad)
            }
            _ => panic!(
                "Data to vectorize not divisible by lanes ({} vs {})",
                V::LANES,
                len,
            ),
        };
        let me = ReadVectorizer {
            start,
            _vector: PhantomData,
            _slice: PhantomData,
        };
        (me, main / V::LANES, partial)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct WriteVectorizer<'a, B, V> {
    start: *mut B,
    _vector: PhantomData<V>,
    _slice: PhantomData<&'a mut [B]>, // To hold the lifetime
}

// Note: The impls here assume V, B, P are Sync and Send, which they are. Nobody is able to create
// this directly and we do have the limits on Vector, the allowed implementations, etc.
unsafe impl<B, V> Send for WriteVectorizer<'_, B, V> {}
unsafe impl<B, V> Sync for WriteVectorizer<'_, B, V> {}

impl<'a, B, V> Vectorizer<MutProxy<'a, B, V>> for WriteVectorizer<'a, B, V>
where
    V: Vector<B>,
    B: Copy,
{
    #[inline]
    unsafe fn get(&self, idx: usize) -> MutProxy<'a, B, V> {
        let ptr = self.start.add(V::LANES * idx);
        MutProxy {
            data: V::new_unchecked(ptr),
            restore: slice::from_raw_parts_mut(ptr, V::LANES),
        }
    }
}

impl<'a, B, V> Vectorizable<MutProxy<'a, B, V>> for &'a mut [B]
where
    B: Copy + 'a,
    V: Vector<B>,
{
    type Vectorizer = WriteVectorizer<'a, B, V>;
    type Padding = V;
    #[inline]
    fn create(self, pad: Option<V>) -> (Self::Vectorizer, usize, Option<MutProxy<'a, B, V>>) {
        let len = self.len();
        assert!(len * mem::size_of::<B>() <= isize::MAX as usize, "Slice too huge");
        let rest = len % V::LANES;
        let main = len - rest;
        let start = self.as_mut_ptr();
        let partial = match (rest, pad) {
            (0, _) => None,
            (_, Some(mut pad)) => {
                let restore = &mut self[main..];
                pad[..rest].copy_from_slice(restore);
                Some(MutProxy {
                    data: pad,
                    restore,
                })
            }
            _ => panic!(
                "Data to vectorize not divisible by lanes ({} vs {})",
                V::LANES,
                len,
            ),
        };
        let me = WriteVectorizer {
            start,
            _vector: PhantomData,
            _slice: PhantomData,
        };
        (me, main / V::LANES, partial)
    }
}

impl<A, B, AR, BR> Vectorizer<(AR, BR)> for (A, B)
where
    A: Vectorizer<AR>,
    B: Vectorizer<BR>,
{
    #[inline]
    unsafe fn get(&self, idx: usize) -> (AR, BR) {
        (self.0.get(idx), self.1.get(idx))
    }
}

impl<A, B, AR, BR> Vectorizable<(AR, BR)> for (A, B)
where
    A: Vectorizable<AR>,
    B: Vectorizable<BR>,
{
    type Vectorizer = (A::Vectorizer, B::Vectorizer);
    type Padding = (A::Padding, B::Padding);
    #[inline]
    fn create(self, pad: Option<Self::Padding>) -> (Self::Vectorizer, usize, Option<(AR, BR)>) {
        let (ap, bp) = if let Some((ap, bp)) = pad {
            (Some(ap), Some(bp))
        } else {
            (None, None)
        };
        let (av, asiz, ap) = self.0.create(ap);
        let (bv, bsiz, bp) = self.1.create(bp);
        // TODO: We may want to support this in the padded mode eventually by creating more
        // paddings
        assert_eq!(asiz, bsiz, "Vectorizing data of different lengths");
        let pad = match (ap, bp) {
            (Some(ap), Some(bp)) => Some((ap, bp)),
            (None, None) => None,
            _ => panic!("Paddings are not provided by both vectorized data"),
        };
        ((av, bv), asiz, pad)
    }
}

// TODO: Macro to generate bigger tuples, we want more than 2 and don't want to do so manually

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iter() {
        let data = (0..=10u16).collect::<Vec<_>>();
        let vtotal: u16x8 = data.vectorize_pad(u16x8::default())
            .sum();
        let total: u16 = vtotal.horizontal_sum();
        assert_eq!(total, 55);
    }

    #[test]
    fn iter_mut() {
        let data = (0..33u32).collect::<Vec<_>>();
        let mut dst = [0u32; 33];
        let ones = u32x4::splat(1);
        for (mut d, s) in (&mut dst[..], &data[..]).vectorize_pad((u32x4::default(), u32x4::default())) {
            *d = ones + s;
        }

        for (l, r) in data.iter().zip(dst.iter()) {
            assert_eq!(*l + 1, *r);
        }
    }
}
