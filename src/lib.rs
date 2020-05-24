#![allow(non_camel_case_types)]

use core::iter;
use core::marker::PhantomData;
use core::mem;
use core::ops::*;

use generic_array::{ArrayLength, GenericArray};
use typenum::marker_traits::Unsigned;

pub mod vector;
pub mod types;

pub use types::*;

pub mod prelude {
    pub use crate::Vector;
    pub use crate::types::*;
}

mod inner {
    use core::num::Wrapping;

    pub trait InstructionSet: Sized { }

    pub unsafe trait Repr<For>: Copy { }

    unsafe impl Repr<u8> for Wrapping<u8> { }
    unsafe impl Repr<u16> for Wrapping<u16> { }
    unsafe impl Repr<u32> for Wrapping<u32> { }
}

// XXX Actually use these (and maybe hide them with impl Iterator)
#[derive(Copy, Clone, Debug)]
pub struct Iter<'a, V, B> {
    data: &'a [B],
    _v: PhantomData<V>,
}

impl<V, B> Iterator for Iter<'_, V, B>
where
    V: Vector<B> + Default,
    B: Copy,
{
    type Item = V;
    #[inline]
    fn next(&mut self) -> Option<V> {
        if self.data.len() >= V::LANES {
            let (start, rest) = self.data.split_at(V::LANES);
            self.data = rest;
            Some(V::new(start))
        } else if self.data.is_empty() {
            None
        } else {
            let mut vec = V::default();
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

struct IterMut<'a, V, B> {
    data: &'a mut [B],
    _v: PhantomData<V>,
}

impl<'a, V, B> Iterator for IterMut<'a, V, B>
where
    V: Vector<B> + Default,
    B: Copy,
{
    type Item = MutProxy<'a, V, B>;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.data.len() >= V::LANES {
            let data = mem::take(&mut self.data);
            let (start, rest) = data.split_at_mut(V::LANES);
            self.data = rest;
            let data = V::new(start);
            Some(MutProxy {
                data,
                restore: start,
            })
        } else if self.data.is_empty() {
            None
        } else {
            let mut data = V::default();
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
struct MutProxy<'a, V, B>
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
    #[inline]
    fn deref(&self) -> &V {
        &self.data
    }
}

impl<V, B> DerefMut for MutProxy<'_, V, B>
where
    V: Deref<Target = [B]>,
    B: Copy,
{
    #[inline]
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

pub trait Vector<B>: Deref<Target = [B]> + DerefMut + Sized + 'static {
    type Lanes: ArrayLength<B>;
    const LANES: usize = Self::Lanes::USIZE;
    // TODO: new_unchecked ‒ aligned, no instruction set checked
    fn new(input: &[B]) -> Self;

    #[inline]
    fn splat(value: B) -> Self
    where
        B: Copy,
    {
        let input = iter::repeat(value)
            .take(Self::LANES)
            .collect::<GenericArray<B, Self::Lanes>>();
        Self::new(&input)
    }
}

#[inline]
pub fn vectorize<'a, B, V>(data: &'a [B]) -> impl Iterator<Item = V> + 'a
where
    B: Copy,
    V: Vector<B> + Default,
{
    Iter {
        data,
        _v: PhantomData,
    }
}

#[inline]
pub fn vectorize_mut<'a, B, V>(data: &'a mut [B])
    -> impl Iterator<Item = impl DerefMut<Target = V> + 'a> + 'a
where
    B: Copy,
    V: Vector<B> + Default,
{
    IterMut {
        data,
        _v: PhantomData,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iter() {
        let data = (0..=10u16).collect::<Vec<_>>();
        let vtotal = vectorize(&data)
            .fold(u16x8::default(), |a, b| a + b);
        let total: u16 = vtotal.iter().sum();
        assert_eq!(total, 55);
    }

    #[test]
    fn iter_mut() {
        let data = (0..33u32).collect::<Vec<_>>();
        let mut dst = [0u32; 33];
        let ones = u32x4::splat(1);
        for (mut d, s) in vectorize_mut(&mut dst).zip(vectorize(&data)) {
            *d = ones + s;
        }

        for (l, r) in data.iter().zip(dst.iter()) {
            assert_eq!(*l + 1, *r);
        }
    }
}
