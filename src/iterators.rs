//! The [`Vectorizable`] trait and a lot of its service types.
//!
//! The [`Vectorizable`] trait allows to turning slices of base types to iterators of vectors, both
//! in separation and in tandem. The rest of this module provides the related types and traits.
//!
//! Usually, it is enough to bring in the [`prelude`][crate::prelude], which already contains the
//! trait. It is seldom necessary to interact with this module directly.
//!
//! # Examples
//!
//! ```rust
//! use slipstream::prelude::*;
//!
//! fn double(input: &[u32], output: &mut [u32]) {
//!     let two = u32x8::splat(2);
//!     for (i, mut o) in (input, output).vectorize() {
//!         *o = two * i;
//!     }
//! }
//! # double(&[], &mut [])
//! ```

use core::iter::FusedIterator;
use core::marker::PhantomData;
use core::mem::{self, MaybeUninit};
use core::ops::*;
use core::ptr;
use core::slice;

use crate::inner::Repr;
use crate::vector::align::Align;
use crate::Vector;

// TODO: Deref to arrays, not slices
/// A proxy object for iterating over mutable slices.
///
/// For technical reasons (mostly alignment and padding), it's not possible to return a simple
/// reference. This type is returned instead and it can be used to both read and write the vectors
/// a slice is turned into.
///
/// Note that the data are written in the destructor. Usually, this should not matter, but if you
/// [`forget`][mem::forget], the changes will be lost (this is meant as a warning, not as a way to
/// implement poor-man's transactions).
#[derive(Debug)]
pub struct MutProxy<'a, B, V>
where
    V: AsRef<[B]>,
    B: Copy,
{
    data: V,
    restore: &'a mut [B],
}

impl<B, V> Deref for MutProxy<'_, B, V>
where
    V: AsRef<[B]>,
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
    V: AsRef<[B]>,
    B: Copy,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut V {
        &mut self.data
    }
}

impl<B, V> Drop for MutProxy<'_, B, V>
where
    V: AsRef<[B]>,
    B: Copy,
{
    #[inline]
    fn drop(&mut self) {
        self.restore
            .copy_from_slice(&self.data.as_ref()[..self.restore.len()]);
    }
}

#[doc(hidden)]
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

#[doc(hidden)]
pub trait Vectorizer<R> {
    /// Get the nth vector.
    ///
    /// # Safety
    ///
    /// * idx must be in range (as declared on creation).
    /// * It may be called at most once per each index.
    unsafe fn get(&mut self, idx: usize) -> R;
}

/// The iterator returned by methods on [`Vectorizable`].
///
/// While it's unusual to need to *name* the type, this is the thing that is returned from
/// [`Vectorizable::vectorize`] and [`Vectorizable::vectorize_pad`]. It might be of interest to
/// know that it implements several iterator „extensions“ ([`DoubleEndedIterator`],
/// [`ExactSizeIterator`] and [`FusedIterator`]). Also, several methods are optimized ‒ for
/// example, the `count` is constant time operation, while the generic is linear.
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
        } else {
            self.partial.take_partial()
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
            Some(unsafe { self.vectorizer.get(self.right) })
        } else {
            None
        }
    }
}

impl<V, P, R> ExactSizeIterator for VectorizedIter<V, P, R>
where
    V: Vectorizer<R>,
    P: Partial<R>,
{
}

impl<V, P, R> FusedIterator for VectorizedIter<V, P, R>
where
    V: Vectorizer<R>,
    P: Partial<R>,
{
}

/// A trait describing things with direct support for splitting into vectors.
///
/// This supports vectorized iteration over shared and mutable slices as well as types composed of
/// them (tuples and short fixed-sized arrays).
///
/// Note that, unlike normal iterators, shared slices return owned values (vectors) and mutable
/// slices return [proxy objects][MutProxy] that allow writing the data back. It is not possible to
/// directly borrow from the slice because of alignment. The tuples and arrays return tuples and
/// arrays of the inner values.
///
/// Already pre-vectorized inputs are also supported (this is useful in combination with other not
/// vectorized inputs).
///
/// # Type hints
///
/// Oftentimes, the compiler can infer the type of the base type, but not the length of the vector.
/// It is therefore needed to provide a type hint.
///
/// Furthermore, for tuples and arrays, the inner type really needs to be the slice, not something
/// that can coerce into it (eg. vec or array).
///
/// Alternatively, you can use the free-standing functions [`vectorize`][crate::vectorize] and
/// [`vectorize_pad`][crate::vectorize_pad]. It allows using the turbofish to provide the hint.
///
/// # Examples
///
/// ```rust
/// # use slipstream::prelude::*;
/// let data = [1, 2, 3, 4];
/// let v = data.vectorize().collect::<Vec<u32x2>>();
/// assert_eq!(vec![u32x2::new([1, 2]), u32x2::new([3, 4])], v);
/// ```
///
/// ```rust
/// # use slipstream::prelude::*;
/// let data = [1, 2, 3, 4];
/// for v in data.vectorize() {
///     let v: u32x2 = v; // Type hint
///     println!("{:?}", v);
/// }
/// ```
///
/// ```rust
/// # use slipstream::prelude::*;
/// let input = [1, 2, 3, 4];
/// let mut output = [0; 4];
/// let mul = u32x2::splat(2);
/// // We have to force the coercion to slice by [..]
/// for (i, mut o) in (&input[..], &mut output[..]).vectorize() {
///     *o = mul * i;
/// }
/// assert_eq!(output, [2, 4, 6, 8]);
/// ```
///
/// ```rust
/// # use slipstream::prelude::*;
/// let vectorized = [u32x2::new([1, 2]), u32x2::new([3, 4])];
/// let not_vectorized = [1, 2, 3, 4];
/// for (v, n) in (&vectorized[..], &not_vectorized[..]).vectorize() {
///     assert_eq!(v, n);
/// }
/// ```
pub trait Vectorizable<V>: Sized {
    /// The input type provided by user to fill in the padding/uneven end.
    ///
    /// Note that this doesn't necessarily have to be the same type as the type returned by the
    /// resulting iterator. For example, in case of mutable slices, the input is the vector, while
    /// the output is [`MutProxy`].
    type Padding;

    /// An internal type managing the splitting into vectors.
    ///
    /// Not of direct interest of the users of this crate.
    type Vectorizer: Vectorizer<V>;

    /// Internal method to create the vectorizer and kick of the iteration.
    fn create(self, pad: Option<Self::Padding>) -> (Self::Vectorizer, usize, Option<V>);

    /// Vectorize a slice or composite of slices
    ///
    /// This variant assumes the input is divisible by the size of the vector. Prefer this if
    /// possible over [`vectorize_pad`][Vectorizable::vectorize_pad], as it is usually
    /// significantly faster.
    ///
    /// # Panics
    ///
    /// * If the slice length isn't divisible by the vector size.
    /// * If the parts of the composite produce different number of vectors. It is not mandated for
    ///   the slices to be of equal length, only to produce the same number of vectors.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use slipstream::prelude::*;
    /// let longer = [1, 2, 3, 4, 5, 6, 7, 8];
    /// let shorter = [1, 2, 3, 4];
    /// for i in (&shorter[..], &longer[..]).vectorize() {
    ///     let (s, l): (u32x2, u32x4) = i;
    ///     println!("s: {:?}, l: {:?})", s, l);
    /// }
    /// ```
    #[inline(always)]
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

    /// Vectorizes a slice or composite of slices, padding the odd end if needed.
    ///
    /// While the [`vectorize`][Vectorizable::vectorize] assumes the input can be split into
    /// vectors without leftover, this version deals with the uneven rest by producing a padding
    /// vector (if needed). The unused lanes are taken from the `pad` parameter. This is at the
    /// cost of some performance (TODO: figure out why it is so much slower).
    ///
    /// For mutable slices, padding is used as usual, but the added lanes are not stored anywhere.
    ///
    /// The padding is produced at the end.
    ///
    /// In case of composites, this still assumes they produce the same number of full vectors and
    /// that they all either do or don't need a padding.
    ///
    /// # Panics
    ///
    /// If the above assumption about number of vectors and same padding behaviour is violated.
    ///
    /// ```rust
    /// # use slipstream::prelude::*;
    /// let data = [1, 2, 3, 4, 5, 6];
    /// let v = data.vectorize_pad(i32x4::splat(-1)).collect::<Vec<_>>();
    /// assert_eq!(v, vec![i32x4::new([1, 2, 3, 4]), i32x4::new([5, 6, -1, -1])]);
    /// ```
    #[inline(always)]
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

#[doc(hidden)]
#[derive(Copy, Clone, Debug)]
pub struct ReadVectorizer<'a, A: Align, B: Repr, const S: usize> {
    start: *const B,
    _vector: PhantomData<Vector<A, B, S>>,
    _slice: PhantomData<&'a [B]>, // To hold the lifetime
}

// Note: The impls here assume V, B, P are Sync and Send, which they are. Nobody is able to create
// this directly and we do have the limits on Vector, the allowed implementations, etc.
unsafe impl<A: Align, B: Repr, const S: usize> Send for ReadVectorizer<'_, A, B, S> {}
unsafe impl<A: Align, B: Repr, const S: usize> Sync for ReadVectorizer<'_, A, B, S> {}

impl<A: Align, B: Repr, const S: usize> Vectorizer<Vector<A, B, S>>
    for ReadVectorizer<'_, A, B, S>
{
    #[inline(always)]
    unsafe fn get(&mut self, idx: usize) -> Vector<A, B, S> {
        Vector::new_unchecked(self.start.add(S * idx))
    }
}

impl<'a, A: Align, B: Repr, const S: usize> Vectorizable<Vector<A, B, S>> for &'a [B] {
    type Vectorizer = ReadVectorizer<'a, A, B, S>;
    type Padding = Vector<A, B, S>;
    #[inline]
    fn create(
        self,
        pad: Option<Vector<A, B, S>>,
    ) -> (Self::Vectorizer, usize, Option<Vector<A, B, S>>) {
        let len = self.len();
        assert!(
            len * mem::size_of::<B>() <= isize::MAX as usize,
            "Slice too huge"
        );
        let rest = len % S;
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
                S, len,
            ),
        };
        let me = ReadVectorizer {
            start,
            _vector: PhantomData,
            _slice: PhantomData,
        };
        (me, main / S, partial)
    }
}

#[doc(hidden)]
#[derive(Copy, Clone, Debug)]
pub struct WriteVectorizer<'a, A: Align, B: Repr, const S: usize> {
    start: *mut B,
    _vector: PhantomData<Vector<A, B, S>>,
    _slice: PhantomData<&'a mut [B]>, // To hold the lifetime
}

// Note: The impls here assume V, B, P are Sync and Send, which they are. Nobody is able to create
// this directly and we do have the limits on Vector, the allowed implementations, etc.
unsafe impl<A: Align, B: Repr, const S: usize> Send for WriteVectorizer<'_, A, B, S> {}
unsafe impl<A: Align, B: Repr, const S: usize> Sync for WriteVectorizer<'_, A, B, S> {}

impl<'a, A: Align, B: Repr, const S: usize> Vectorizer<MutProxy<'a, B, Vector<A, B, S>>>
    for WriteVectorizer<'a, A, B, S>
{
    #[inline(always)]
    unsafe fn get(&mut self, idx: usize) -> MutProxy<'a, B, Vector<A, B, S>> {
        // FIXME: Technically, we extend the lifetime in the from_raw_parts_mut beyond what rust
        // would allow us to normally do. But is this OK? As we are guaranteed never to give any
        // chunk twice, this should act similar to IterMut from slice or similar.
        let ptr = self.start.add(S * idx);
        MutProxy {
            data: Vector::new_unchecked(ptr),
            restore: slice::from_raw_parts_mut(ptr, S),
        }
    }
}

impl<'a, A: Align, B: Repr, const S: usize> Vectorizable<MutProxy<'a, B, Vector<A, B, S>>>
    for &'a mut [B]
{
    type Vectorizer = WriteVectorizer<'a, A, B, S>;
    type Padding = Vector<A, B, S>;
    #[inline]
    #[allow(clippy::type_complexity)]
    fn create(
        self,
        pad: Option<Vector<A, B, S>>,
    ) -> (
        Self::Vectorizer,
        usize,
        Option<MutProxy<'a, B, Vector<A, B, S>>>,
    ) {
        let len = self.len();
        assert!(
            len * mem::size_of::<B>() <= isize::MAX as usize,
            "Slice too huge"
        );
        let rest = len % S;
        let main = len - rest;
        let start = self.as_mut_ptr();
        let partial = match (rest, pad) {
            (0, _) => None,
            (_, Some(mut pad)) => {
                let restore = &mut self[main..];
                pad[..rest].copy_from_slice(restore);
                Some(MutProxy { data: pad, restore })
            }
            _ => panic!(
                "Data to vectorize not divisible by lanes ({} vs {})",
                S, len,
            ),
        };
        let me = WriteVectorizer {
            start,
            _vector: PhantomData,
            _slice: PhantomData,
        };
        (me, main / S, partial)
    }
}

macro_rules! vectorizable_tuple {
    ($(($X: ident, $XR: ident, $X0: tt)),*) => {
        impl<$($X, $XR),*> Vectorizer<($($XR),*)> for ($($X),*)
        where
            $($X: Vectorizer<$XR>,)*
        {
            #[inline(always)]
            unsafe fn get(&mut self, idx: usize) -> ($($XR),*) {
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
            #[allow(clippy::eq_op)]
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
vectorizable_tuple!(
    (A, AR, 0),
    (B, BR, 1),
    (C, CR, 2),
    (D, DR, 3),
    (E, ER, 4),
    (F, FR, 5)
);
vectorizable_tuple!(
    (A, AR, 0),
    (B, BR, 1),
    (C, CR, 2),
    (D, DR, 3),
    (E, ER, 4),
    (F, FR, 5),
    (G, GR, 6)
);
vectorizable_tuple!(
    (A, AR, 0),
    (B, BR, 1),
    (C, CR, 2),
    (D, DR, 3),
    (E, ER, 4),
    (F, FR, 5),
    (G, GR, 6),
    (H, HR, 7)
);

impl<T, TR, const S: usize> Vectorizer<[TR; S]> for [T; S]
where
    T: Vectorizer<TR>,
{
    #[inline(always)]
    unsafe fn get(&mut self, idx: usize) -> [TR; S] {
        let mut res = MaybeUninit::<[TR; S]>::uninit();
        for (i, v) in self.iter_mut().enumerate() {
            ptr::write(res.as_mut_ptr().cast::<TR>().add(i), v.get(idx));
        }
        res.assume_init()
    }
}

impl<T, TR, const S: usize> Vectorizable<[TR; S]> for [T; S]
where
    T: Vectorizable<TR> + Copy,
    T::Padding: Copy,
{
    type Vectorizer = [T::Vectorizer; S];
    type Padding = [T::Padding; S];
    #[inline]
    fn create(self, pad: Option<Self::Padding>) -> (Self::Vectorizer, usize, Option<[TR; S]>) {
        let mut vectorizer = MaybeUninit::<Self::Vectorizer>::uninit();
        let mut size = 0;
        let mut padding = MaybeUninit::<[TR; S]>::uninit();
        let mut seen_some_pad = false;
        let mut seen_none_pad = false;
        unsafe {
            for i in 0..S {
                let (v, s, p) = self[i].create(pad.map(|p| p[i]));
                ptr::write(vectorizer.as_mut_ptr().cast::<T::Vectorizer>().add(i), v);
                if i == 0 {
                    size = s;
                } else {
                    assert_eq!(size, s, "Vectorized lengths inconsistent across the array",);
                }
                match p {
                    Some(p) => {
                        seen_some_pad = true;
                        ptr::write(padding.as_mut_ptr().cast::<TR>().add(i), p);
                    }
                    None => seen_none_pad = true,
                }
            }
            assert!(
                !seen_some_pad || !seen_none_pad,
                "Paddings inconsistent across the array",
            );
            let padding = if seen_some_pad {
                Some(padding.assume_init())
            } else {
                None
            };
            (vectorizer.assume_init(), size, padding)
        }
    }
}

impl<'a, T> Vectorizer<T> for &'a [T]
where
    T: Copy,
{
    unsafe fn get(&mut self, idx: usize) -> T {
        *self.get_unchecked(idx)
    }
}

impl<'a, T> Vectorizer<&'a mut T> for &'a mut [T] {
    unsafe fn get(&mut self, idx: usize) -> &'a mut T {
        // FIXME: Why do we have to extend the lifetime here? Is it safe? Intuitively, it should,
        // because we hand out each chunk only once and this is what IterMut does too.
        let ptr = self.get_unchecked_mut(idx) as *mut T;
        &mut *ptr
    }
}

impl<'a, A: Align, B: Repr, const S: usize> Vectorizable<Vector<A, B, S>>
    for &'a [Vector<A, B, S>]
{
    type Padding = ();
    type Vectorizer = &'a [Vector<A, B, S>];
    fn create(self, _pad: Option<()>) -> (Self::Vectorizer, usize, Option<Vector<A, B, S>>) {
        (self, self.len(), None)
    }
}

impl<'a, A: Align, B: Repr, const S: usize> Vectorizable<&'a mut Vector<A, B, S>>
    for &'a mut [Vector<A, B, S>]
{
    type Padding = ();
    type Vectorizer = &'a mut [Vector<A, B, S>];
    fn create(
        self,
        _pad: Option<()>,
    ) -> (Self::Vectorizer, usize, Option<&'a mut Vector<A, B, S>>) {
        let len = self.len();
        (self, len, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;

    #[test]
    fn iter() {
        let data = (0..=10u16).collect::<Vec<_>>();
        let vtotal: u16x8 = data.vectorize_pad(u16x8::default()).sum();
        let total: u16 = vtotal.horizontal_sum();
        assert_eq!(total, 55);
    }

    #[test]
    fn iter_mut() {
        let data = (0..33u32).collect::<Vec<_>>();
        let mut dst = [0u32; 33];
        let ones = u32x4::splat(1);
        for (mut d, s) in
            (&mut dst[..], &data[..]).vectorize_pad((u32x4::default(), u32x4::default()))
        {
            *d = ones + s;
        }

        for (l, r) in data.iter().zip(dst.iter()) {
            assert_eq!(*l + 1, *r);
        }
    }

    // Here, one of the inputs is already vectorized
    #[test]
    fn iter_prevec() {
        let src = [0, 1, 2, 3, 4, 5, 6, 7];
        let mut dst = [u16x4::default(); 2];

        for (dst, src) in (&mut dst[..], &src[..]).vectorize() {
            *dst = src;
        }

        assert_eq!(dst, [u16x4::new([0, 1, 2, 3]), u16x4::new([4, 5, 6, 7])]);
    }
}
