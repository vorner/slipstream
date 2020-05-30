#![allow(non_camel_case_types)]

use core::iter;
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
pub fn vectorize<'a, V, B>(data: &'a [B], mut empty: V) -> impl Iterator<Item = V> + 'a
where
    B: Copy,
    V: Vector<B> + Default,
{
    let chunks = data.chunks_exact(V::LANES);
    let last = chunks.remainder();
    let last = if last.is_empty() {
        None
    } else {
        empty[0..last.len()].copy_from_slice(last);
        Some(empty)
    };
    chunks.map(V::new).chain(last)
}

#[inline]
pub fn vectorize_exact<'a, V, B>(data: &'a [B]) -> impl Iterator<Item = V> + 'a
where
    B: Copy,
    V: Vector<B> + Default,
{
    assert!(
        data.len() % V::LANES == 0,
        "Data to vectorize_exact must be divisible by number of lanes ({} % {})",
        data.len(), V::LANES,
    );
    data.chunks_exact(V::LANES).map(V::new)
}

#[inline]
pub fn vectorize_mut<'a, V, B>(mut data: &'a mut [B], mut empty: V)
    -> impl Iterator<Item = impl DerefMut<Target = V> + 'a> + 'a
where
    B: Copy,
    V: Vector<B> + Default,
{
    let rem = data.len() % V::LANES;
    let mut last = None;
    if rem > 0 {
        let (d, r) = data.split_at_mut(data.len() - rem);
        data = d;
        empty[0..rem].copy_from_slice(r);
        last = Some(MutProxy {
            data: empty,
            restore: r,
        });
    }

    vectorize_mut_inner(data).chain(last)
}

#[inline]
fn vectorize_mut_inner<'a, V, B>(data: &'a mut [B])
    -> impl Iterator<Item = MutProxy<'a, V, B>> + 'a
where
    B: Copy,
    V: Vector<B> + Default,
{
    data
        .chunks_exact_mut(V::LANES)
        .map(|d| MutProxy {
            data: V::new(d),
            restore: d,
        })
}

#[inline]
pub fn vectorize_mut_exact<'a, V, B>(data: &'a mut [B])
    -> impl Iterator<Item = impl DerefMut<Target = V> + 'a> + 'a
where
    B: Copy,
    V: Vector<B> + Default,
{
    assert!(
        data.len() % V::LANES == 0,
        "Data to vectorize_exact must be divisible by number of lanes ({} % {})",
        data.len(), V::LANES,
    );
    vectorize_mut_inner(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iter() {
        let data = (0..=10u16).collect::<Vec<_>>();
        let vtotal = vectorize(&data, u16x8::default())
            .fold(u16x8::default(), |a, b| a + b);
        let total: u16 = vtotal.iter().sum();
        assert_eq!(total, 55);
    }

    #[test]
    fn iter_mut() {
        let data = (0..33u32).collect::<Vec<_>>();
        let mut dst = [0u32; 33];
        let ones = u32x4::splat(1);
        for (mut d, s) in vectorize_mut(&mut dst, u32x4::default()).zip(vectorize(&data, u32x4::default())) {
            *d = ones + s;
        }

        for (l, r) in data.iter().zip(dst.iter()) {
            assert_eq!(*l + 1, *r);
        }
    }
}
