use core::iter::FusedIterator;
use core::marker::PhantomData;
use core::mem;
use core::ops::*;
use core::slice;

use generic_array::ArrayLength;
use crate::{inner, Vector};

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

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.right - self.left + self.partial.size();
        (len, Some(len))
    }

    // Overriden for performance… these things have no side effects, so we can avoid calling next

    #[inline]
    fn count(self) -> usize {
        self.size_hint().0
    }

    #[inline]
    fn last(mut self) -> Option<R> {
        self.next_back()
    }

    // TODO: This wants some tests
    #[inline]
    fn nth(&mut self, n: usize) -> Option<R> {
        let main_len = self.right - self.left;
        if main_len >= n {
            self.left += n;
            self.next()
        } else {
            self.left = self.right;
            self.partial.take_partial();
            None
        }
    }
}

impl<V, P, R> DoubleEndedIterator for VectorizedIter<V, P, R>
where
    V: Vectorizer<R>,
    P: Partial<R>,
{
    // TODO: Tests
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if let Some(partial) = self.partial.take_partial() {
            Some(partial)
        } else if self.left < self.right {
            self.right -= 1;
            Some(unsafe { self.vectorizer.get(self.right)})
        } else {
            None
        }
    }
}

impl<V, P, R> ExactSizeIterator for VectorizedIter<V, P, R>
where
    V: Vectorizer<R>,
    P: Partial<R>,
{ }

impl<V, P, R> FusedIterator for VectorizedIter<V, P, R>
where
    V: Vectorizer<R>,
    P: Partial<R>,
{ }

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
    B: inner::Repr,
    V: Vector<Base = B>,
    V::Lanes: ArrayLength<B>,
{
    #[inline]
    unsafe fn get(&self, idx: usize) -> V {
        V::new_unchecked(self.start.add(V::LANES * idx))
    }
}

impl<'a, B, V> Vectorizable<V> for &'a [B]
where
    B: inner::Repr,
    V: Vector<Base = B> + Deref<Target = [B]> + DerefMut,
    V::Lanes: ArrayLength<B>,
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
    B: inner::Repr,
    V: Vector<Base = B> + Deref<Target = [B]> + DerefMut,
    V::Lanes: ArrayLength<B>,
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
    B: inner::Repr,
    V: Vector<Base = B> + Deref<Target = [B]> + DerefMut,
    V::Lanes: ArrayLength<B>,
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

macro_rules! vectorizable_tuple {
    ($(($X: ident, $XR: ident, $X0: tt)),*) => {
        impl<$($X, $XR),*> Vectorizer<($($XR),*)> for ($($X),*)
        where
            $($X: Vectorizer<$XR>,)*
        {
            #[inline]
            unsafe fn get(&self, idx: usize) -> ($($XR),*) {
                ($(self.$X0.get(idx)),*)
            }
        }

        impl<$($X, $XR),*> Vectorizable<($($XR),*)> for ($($X),*)
        where
            $($X: Vectorizable<$XR>,)*
        {
            type Vectorizer = ($($X::Vectorizer),*);
            type Padding = ($($X::Padding),*);
            #[inline]
            fn create(self, pad: Option<Self::Padding>)
                -> (Self::Vectorizer, usize, Option<($($XR),*)>)
            {
                let pad = match pad {
                    Some(pad) => ($(Some(pad.$X0)),*),
                    None => Default::default(), // Bunch of Nones in a tuple.. (None, None, None)...
                };
                let created = ($(self.$X0.create(pad.$X0)),*);
                $(
                    // TODO: We may want to support this in the padded mode eventually by
                    // creating more paddings
                    assert_eq!(
                        (created.0).1,
                        created.$X0.1,
                        "Vectorizing data of different lengths"
                    );
                    // TODO: We could also handle this in the padded mode by doing empty pads
                    assert_eq!(
                        (created.0).2.is_some(),
                        created.$X0.2.is_some(),
                        "Paddings are not the same for all vectorized data",
                    );
                )*
                let vectorizer = ($(created.$X0.0),*);
                let pad = if (created.0).2.is_some() {
                    Some(($(created.$X0.2.unwrap()),*))
                } else {
                    None
                };
                (vectorizer, (created.0).1, pad)
            }
        }
    }
}

vectorizable_tuple!((A, AR, 0), (B, BR, 1));
vectorizable_tuple!((A, AR, 0), (B, BR, 1), (C, CR, 2));
vectorizable_tuple!((A, AR, 0), (B, BR, 1), (C, CR, 2), (D, DR, 3));
vectorizable_tuple!((A, AR, 0), (B, BR, 1), (C, CR, 2), (D, DR, 3), (E, ER, 4));
vectorizable_tuple!((A, AR, 0), (B, BR, 1), (C, CR, 2), (D, DR, 3), (E, ER, 4), (F, FR, 5));
vectorizable_tuple!((A, AR, 0), (B, BR, 1), (C, CR, 2), (D, DR, 3), (E, ER, 4), (F, FR, 5), (G, GR, 6));
vectorizable_tuple!((A, AR, 0), (B, BR, 1), (C, CR, 2), (D, DR, 3), (E, ER, 4), (F, FR, 5), (G, GR, 6), (H, HR, 7));

#[cfg(test)]
mod tests {
    use crate::prelude::*;
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
