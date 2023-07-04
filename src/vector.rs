//! Low-level definitions of the vector types and their traits.
//!
//! While the user usually operates with the type aliases defined in [`types`][crate::types] (and
//! exported through the [`prelude`][crate::prelude], this module provides the actual
//! implementation of the types.
//!
//! The module defines a [`Vector`] type. This allows setting not only the base type and number of
//! lanes, but also alignment (through an additional alignment marker type, available in the
//! [`align`][mod@align] submodule).
//!
//! There are multiple alignments available. Small vectors shouldn't require bigger alignment than
//! their size, while the bigger ones should require larger one to make it possible to use wider
//! SIMD registers.
//!
//! The type aliases in [`types`][crate::types] takes this into account.
//!
//! These types aliases are not thoroughly documented on themselves. The documentation is on the
//! [`Vector`]. A lot of its functionality is in traits it implements.

use core::fmt::{Debug, Formatter, Result as FmtResult};
use core::iter::{Product, Sum};
use core::mem::{self, MaybeUninit};
use core::ops::*;
use core::ptr;
use num_traits::Float;

use self::align::Align;
use crate::inner::Repr;
use crate::Mask;

/// Enforcement of alignment.
///
/// This is mostly an implementation detail seldom used by consumers of the crate.
pub mod align {
    /// Marker trait for alignment enforcers.
    ///
    /// The SIMD vectors need to be properly aligned. Rust allows doing that by an attribute, but that
    /// needs another top-level vector type. We use zero-sized types to enforce it in a different way.
    ///
    /// This is just a marker type for the enforcers, to avoid people putting the wrong parameter at
    /// the wrong place.
    pub trait Align: Copy {}

    macro_rules! align {
        ($name: ident, $align: expr) => {
            /// Alignment marker.
            #[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
            #[repr(align($align))]
            pub struct $name;
            impl Align for $name {}
        };
    }

    align!(Align1, 1);
    align!(Align2, 2);
    align!(Align4, 4);
    align!(Align8, 8);
    align!(Align16, 16);
    align!(Align32, 32);
    align!(Align64, 64);
    align!(Align128, 128);
}

// TODO: Seal?
/// Trait to look up a mask corresponding to a type.
///
/// The [`Vector`] implements this and allows for finding out what the corresponding mask type for
/// it is. This is not an inherent associated type because these don't yet exist in Rust.
pub trait Masked {
    /// The mask type for this vector.
    ///
    /// Masks are vector types of boolean-like base types. They are used as results of lane-wise
    /// comparisons like [`eq`][Vector::eq] and for enabling subsets of lanes for certain
    /// operations, like [`blend`][Vector::blend] and
    /// [`gather_load_masked`][Vector::gather_load_masked].
    ///
    /// This associated type describes the native mask for the given vector. For example for
    /// [`u32x4`][crate::u32x4] it would be [`m32x4`][crate::m32x4]. This is the type that the
    /// comparisons produce. While the selection methods accept any mask type of the right number
    /// of lanes, using this type on their input is expected to yield the best performance.
    type Mask;
}

macro_rules! bin_op_impl {
    ($tr: ident, $meth: ident, $tr_assign: ident, $meth_assign: ident) => {
        impl<A: Align, B: $tr<Output = B> + Repr, const S: usize> $tr for Vector<A, B, S> {
            type Output = Self;
            #[inline]
            fn $meth(self, rhs: Self) -> Self {
                unsafe {
                    let mut data = MaybeUninit::<Self>::uninit();
                    for i in 0..S {
                        ptr::write(
                            data.as_mut_ptr().cast::<B>().add(i),
                            $tr::$meth(self.data[i], rhs.data[i]),
                        );
                    }
                    data.assume_init()
                }
            }
        }

        impl<A: Align, B: $tr<Output = B> + Repr, const S: usize> $tr<B> for Vector<A, B, S> {
            type Output = Self;
            #[inline]
            fn $meth(self, rhs: B) -> Self {
                unsafe {
                    let mut data = MaybeUninit::<Self>::uninit();
                    for i in 0..S {
                        ptr::write(
                            data.as_mut_ptr().cast::<B>().add(i),
                            $tr::$meth(self.data[i], rhs),
                        );
                    }
                    data.assume_init()
                }
            }
        }

        impl<A: Align, B: $tr_assign + Repr, const S: usize> $tr_assign for Vector<A, B, S> {
            #[inline]
            fn $meth_assign(&mut self, rhs: Self) {
                for i in 0..S {
                    $tr_assign::$meth_assign(&mut self.data[i], rhs.data[i]);
                }
            }
        }

        impl<A: Align, B: $tr_assign + Repr, const S: usize> $tr_assign<B> for Vector<A, B, S> {
            #[inline]
            fn $meth_assign(&mut self, rhs: B) {
                for i in 0..S {
                    $tr_assign::$meth_assign(&mut self.data[i], rhs);
                }
            }
        }
    };
}

macro_rules! una_op_impl {
    ($tr: ident, $meth: ident) => {
        impl<A: Align, B: $tr<Output = B> + Repr, const S: usize> $tr for Vector<A, B, S> {
            type Output = Self;
            #[inline]
            fn $meth(self) -> Self {
                unsafe {
                    let mut data = MaybeUninit::<Self>::uninit();
                    for i in 0..S {
                        ptr::write(
                            data.as_mut_ptr().cast::<B>().add(i),
                            $tr::$meth(self.data[i]),
                        );
                    }
                    data.assume_init()
                }
            }
        }
    };
}

macro_rules! cmp_op {
    ($($(#[ $meta: meta ])* $tr: ident => $op: ident;)*) => {
        $(
            $(#[ $meta ])*
            #[inline]
            pub fn $op(self, other: Self) -> <Self as Masked>::Mask
            where
                B: $tr,
            {
                let mut data = MaybeUninit::<<Self as Masked>::Mask>::uninit();
                unsafe {
                    for i in 0..S {
                        ptr::write(
                            data.as_mut_ptr().cast::<B::Mask>().add(i),
                            B::Mask::from_bool(self.data[i].$op(&other.data[i])),
                        );
                    }
                    data.assume_init()
                }
            }
        )*
    };
}

/// A vector type.
///
/// Vector types are mostly well aligned fixed sized arrays. Unlike the arrays, they have the usual
/// numeric operators and several helpful methods implemented on them. They perform the operations
/// „per lane“ independently and allow the CPU to parallelize the computations.
///
/// The types have convenient aliases ‒ for example [`u32x4`][crate::u32x4] is an alias for
/// `Vector<Align16, u32, 4>` and corresponds to `[u32; 4]` (but aligned to 16 bytes).
///
/// While these can be operated as arrays (indexing, copying between slices, etc), it is better to
/// perform operations on whole vectors at once.
///
/// The usual comparing operators don't exist (`<=`), but there are „per lane“ comparison operators
/// that return mask vectors ‒ vectors of boolean-like values. These can either be examined
/// manually, or fed into other operations on vectors, like [`blend`][Vector::blend] or
/// [`gather_load_masked`][Vector::gather_load_masked].
///
/// # Examples
///
/// ```rust
/// # use slipstream::prelude::*;
/// let a = i32x4::new([1, -2, 3, -4]);
/// let b = -a;                           // [-1, 2, -3, 4]
/// let positive = a.ge(i32x4::splat(1)); // Lane-wise a >= 1
/// // Will take from b where positive is true, from a otherwise
/// let abs = b.blend(a, positive);
/// assert_eq!(abs, i32x4::new([1, 2, 3, 4]));
/// ```
#[repr(C)]
#[derive(Copy, Clone)]
pub struct Vector<A, B, const S: usize>
where
    A: Align,
    B: Repr,
{
    _align: [A; 0],
    data: [B; S],
}

impl<A, B, const S: usize> Vector<A, B, S>
where
    A: Align,
    B: Repr,
{
    /// Number of lanes of the vector.
    pub const LANES: usize = S;

    #[inline(always)]
    fn assert_size() {
        assert!(S > 0);
        assert!(
            isize::MAX as usize > mem::size_of::<Self>(),
            "Vector type too huge",
        );
        assert_eq!(
            mem::size_of::<Self>(),
            mem::size_of::<[B; S]>(),
            "Must not contain paddings/invalid Align parameter",
        );
    }

    /// Loads the vector without doing bounds checks.
    ///
    /// # Safety
    ///
    /// The pointed to memory must be valid in `Self::LANES` consecutive cells ‒ eg. it must
    /// contain a full array of the base types.
    #[inline]
    pub unsafe fn new_unchecked(input: *const B) -> Self {
        Self::assert_size();
        Self {
            _align: [],
            data: ptr::read(input.cast()),
        }
    }
    /// Loads the vector from correctly sized slice.
    ///
    /// This loads the vector from correctly sized slice or anything that can be converted to it ‒
    /// specifically, fixed sized arrays and other vectors work.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use slipstream::prelude::*;
    /// let vec = (0..10).collect::<Vec<_>>();
    /// let v1 = u32x4::new(&vec[0..4]);
    /// let v2 = u32x4::new(v1);
    /// let v3 = u32x4::new([2, 3, 4, 5]);
    /// assert_eq!(v1 + v2 + v3, u32x4::new([2, 5, 8, 11]));
    /// ```
    ///
    /// # Panics
    ///
    /// If the provided slice is of incompatible size.
    #[inline]
    pub fn new<I>(input: I) -> Self
    where
        I: AsRef<[B]>,
    {
        let input = input.as_ref();
        assert_eq!(
            input.len(),
            S,
            "Creating vector from the wrong sized slice (expected {}, got {})",
            S,
            input.len(),
        );
        unsafe { Self::new_unchecked(input.as_ptr()) }
    }

    // TODO: Can we turn it into const fn?
    /// Produces a vector of all lanes set to the same value.
    ///
    /// ```rust
    /// # use slipstream::prelude::*;
    /// let v = f32x4::splat(1.2);
    /// assert_eq!(v, f32x4::new([1.2, 1.2, 1.2, 1.2]));
    /// ```
    #[inline]
    pub fn splat(value: B) -> Self {
        Self::assert_size();
        Self {
            _align: [],
            data: [value; S],
        }
    }

    /// Loads the vector from a slice by indexing it.
    ///
    /// Unlike [`new`], this can load the vector from discontinuous parts of the slice, out of
    /// order or multiple lanes from the same location. This flexibility comes at the cost of lower
    /// performance (in particular, I've never seen this to get auto-vectorized even though a
    /// gather instruction exists), therefore prefer [`new`] where possible.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use slipstream::prelude::*;
    /// let input = (2..100).collect::<Vec<_>>();
    /// let vec = u32x4::gather_load(&input, [3, 3, 1, 32]);
    /// assert_eq!(vec, u32x4::new([5, 5, 3, 34]));
    /// ```
    ///
    /// It is possible to use another vector as the indices:
    ///
    /// ```rust
    /// # use slipstream::prelude::*;
    /// let indices = usizex4::new([1, 2, 3, 4]) * usizex4::splat(2);
    /// let input = (0..10).collect::<Vec<_>>();
    /// let vec = u32x4::gather_load(&input, indices);
    /// assert_eq!(vec, u32x4::new([2, 4, 6, 8]));
    /// ```
    ///
    /// It is possible to use another vector as an input, allowing to narrow it down or shuffle.
    ///
    /// ```rust
    /// # use slipstream::prelude::*;
    /// let a = u32x4::new([1, 2, 3, 4]);
    /// let b = u32x4::gather_load(a, [2, 0, 1, 3]);
    /// assert_eq!(b, u32x4::new([3, 1, 2, 4]));
    /// let c = u32x2::gather_load(a, [2, 2]);
    /// assert_eq!(c, u32x2::new([3, 3]));
    /// ```
    ///
    /// # Panics
    ///
    /// * If the `idx` slice doesn't have the same length as the vector.
    /// * If any of the indices is out of bounds of the `input`.
    ///
    /// [`new`]: Vector::new
    #[inline]
    pub fn gather_load<I, Idx>(input: I, idx: Idx) -> Self
    where
        I: AsRef<[B]>,
        Idx: AsRef<[usize]>,
    {
        Self::assert_size();
        let input = input.as_ref();
        let idx = idx.as_ref();
        assert_eq!(
            S,
            idx.len(),
            "Gathering vector from wrong number of indexes"
        );
        assert!(idx.iter().all(|&l| l < input.len()), "Gather out of bounds");
        let mut data = MaybeUninit::<Self>::uninit();
        unsafe {
            for i in 0..S {
                let idx = *idx.get_unchecked(i);
                let input = *input.get_unchecked(idx);
                ptr::write(data.as_mut_ptr().cast::<B>().add(i), input);
            }
            data.assume_init()
        }
    }

    /// Loads enabled lanes from a slice by indexing it.
    ///
    /// This is similar to [`gather_load`]. However, the loading of lanes is
    /// enabled by a mask. If the corresponding lane mask is not set, the value is taken from
    /// `self`. In other words, if the mask is all-true, it is semantically equivalent to
    /// [`gather_load`], expect with possible worse performance.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use slipstream::prelude::*;
    /// let input = (0..100).collect::<Vec<_>>();
    /// let v = u32x4::default().gather_load_masked(
    ///     &input,
    ///     [1, 4, 2, 2],
    ///     [m32::TRUE, m32::FALSE, m32::FALSE, m32::TRUE]
    /// );
    /// assert_eq!(v, u32x4::new([1, 0, 0, 2]));
    /// ```
    ///
    /// ```rust
    /// # use slipstream::prelude::*;
    /// let left = u32x2::new([1, 2]);
    /// let right = u32x2::new([3, 4]);
    /// let idx = usizex4::new([0, 1, 0, 1]);
    /// let mask = m32x4::new([m32::TRUE, m32::TRUE, m32::FALSE, m32::FALSE]);
    /// let v = u32x4::default()
    ///     .gather_load_masked(left, idx, mask)
    ///     .gather_load_masked(right, idx, !mask);
    /// assert_eq!(v, u32x4::new([1, 2, 3, 4]));
    /// ```
    ///
    /// # Panics
    ///
    /// * If the `mask` or the `idx` parameter is of different length than the vector.
    /// * If any of the active indices are out of bounds of `input`.
    ///
    /// [`gather_load`]: Vector::gather_load
    #[inline]
    pub fn gather_load_masked<I, Idx, M, MB>(mut self, input: I, idx: Idx, mask: M) -> Self
    where
        I: AsRef<[B]>,
        Idx: AsRef<[usize]>,
        M: AsRef<[MB]>,
        MB: Mask,
    {
        let input = input.as_ref();
        let idx = idx.as_ref();
        let mask = mask.as_ref();
        let len = idx.len();
        assert_eq!(S, len, "Gathering vector from wrong number of indexes");
        assert_eq!(S, mask.len(), "Gathering with wrong sized mask");
        for i in 0..S {
            unsafe {
                if mask.get_unchecked(i).bool() {
                    let idx = *idx.get_unchecked(i);
                    self[i] = input[idx];
                }
            }
        }
        self
    }

    /// Stores the content into a continuous slice of the correct length.
    ///
    /// This is less general than [`scatter_store`][Vector::scatter_store], that one allows storing
    /// to different parts of the slice.
    ///
    /// The counterpart of this is [`new`][Vector::new].
    ///
    /// # Panics
    ///
    /// If the length doesn't match.
    #[inline]
    pub fn store<O: AsMut<[B]>>(self, mut output: O) {
        output.as_mut().copy_from_slice(&self[..])
    }

    /// Store the vector into a slice by indexing it.
    ///
    /// This is the inverse of [`gather_load`][Vector::gather_load]. It takes the lanes of the
    /// vector and stores them into the slice into given indices.
    ///
    /// If you want to store it into a continuous slice, it is potentially faster to do it using
    /// the `copy_from_slice` method or by [`store`][Vector::store]:
    ///
    /// ```rust
    /// # use slipstream::prelude::*;
    /// let mut data = vec![0; 6];
    /// let v = u32x4::new([1, 2, 3, 4]);
    /// data[0..4].copy_from_slice(&v[..]);
    /// assert_eq!(&data[..], &[1, 2, 3, 4, 0, 0]);
    /// v.store(&mut data[..4]);
    /// assert_eq!(&data[..], &[1, 2, 3, 4, 0, 0]);
    /// ```
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use slipstream::prelude::*;
    /// let mut data = vec![0; 6];
    /// let v = u32x4::new([1, 2, 3, 4]);
    /// v.scatter_store(&mut data, [2, 5, 0, 1]);
    /// assert_eq!(&data[..], &[3, 4, 1, 0, 0, 2]);
    /// ```
    ///
    /// # Warning
    ///
    /// If multiple lanes are to be stored into the same slice element, it is not specified which
    /// of them will end up being stored. It is not UB to do so and it'll always be one of them,
    /// however it may change between versions or even between compilation targets which.
    ///
    /// This is to allow for potential different behaviour of different platforms.
    ///
    /// # Panics
    ///
    /// * If the `idx` has a different length than the vector.
    /// * If any of the indices are out of bounds of `output`.
    #[inline]
    pub fn scatter_store<O, Idx>(self, mut output: O, idx: Idx)
    where
        O: AsMut<[B]>,
        Idx: AsRef<[usize]>,
    {
        let output = output.as_mut();
        let idx = idx.as_ref();
        assert_eq!(S, idx.len(), "Scattering vector to wrong number of indexes");
        // Check prior to starting the scatter before we write anything. Might be nicer for
        // optimizer + we don't want to do partial scatter.
        assert!(
            idx.iter().all(|&l| l < output.len()),
            "Scatter out of bounds"
        );
        for i in 0..S {
            unsafe {
                // get_unchecked: index checked above in bulk and we use this one in hope
                // it'll taste better to the autovectorizer and it might find a scatter
                // insrtuction for us.
                let idx = *idx.get_unchecked(i);
                *output.get_unchecked_mut(idx) = self[i];
            }
        }
    }

    /// A masked version of [`scatter_store`].
    ///
    /// This acts in the same way as [`scatter_store`], except lanes disabled by the `mask` are not
    /// stored anywhere.
    ///
    /// # Panics
    ///
    /// * If the `idx` or `mask` has a different length than the vector.
    /// * If any of the active indices are out of bounds of `output`.
    ///
    /// [`scatter_store`]: Vector::scatter_store
    #[inline]
    pub fn scatter_store_masked<O, Idx, M, MB>(self, mut output: O, idx: Idx, mask: M)
    where
        O: AsMut<[B]>,
        Idx: AsRef<[usize]>,
        M: AsRef<[MB]>,
        MB: Mask,
    {
        let output = output.as_mut();
        let idx = idx.as_ref();
        let mask = mask.as_ref();
        assert_eq!(S, idx.len(), "Scattering vector to wrong number of indexes");
        assert_eq!(S, mask.len(), "Scattering vector with wrong sized mask");
        // Check prior to starting the scatter before we write anything. Might be nicer for
        // optimizer + we don't want to do partial scatter.
        let in_bounds = idx
            .iter()
            .enumerate()
            .all(|(i, &l)| !mask[i].bool() || l < output.len());
        assert!(in_bounds, "Scatter out of bounds");
        for i in 0..S {
            if mask[i].bool() {
                unsafe {
                    // get_unchecked: index checked above in bulk and we use this one in
                    // hope it'll taste better to the autovectorizer and it might find a
                    // scatter insrtuction for us.
                    let idx = *idx.get_unchecked(i);
                    *output.get_unchecked_mut(idx) = self[i];
                }
            }
        }
    }

    /// Blend self and other using mask.
    ///
    /// Imports enabled lanes from `other`, keeps disabled lanes from `self`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use slipstream::prelude::*;
    /// let odd = u32x4::new([1, 3, 5, 7]);
    /// let even = u32x4::new([2, 4, 6, 8]);
    /// let mask = m32x4::new([m32::TRUE, m32::FALSE, m32::TRUE, m32::FALSE]);
    /// assert_eq!(odd.blend(even, mask), u32x4::new([2, 3, 6, 7]));
    /// ```
    #[inline]
    pub fn blend<M, MB>(self, other: Self, mask: M) -> Self
    where
        M: AsRef<[MB]>,
        MB: Mask,
    {
        let mut data = MaybeUninit::<Self>::uninit();
        let mask = mask.as_ref();
        unsafe {
            for i in 0..S {
                ptr::write(
                    data.as_mut_ptr().cast::<B>().add(i),
                    if mask[i].bool() { other[i] } else { self[i] },
                );
            }
            data.assume_init()
        }
    }

    /// A lane-wise maximum.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use slipstream::prelude::*;
    /// let a = u32x4::new([1, 4, 2, 5]);
    /// let b = u32x4::new([2, 3, 2, 6]);
    /// assert_eq!(a.maximum(b), u32x4::new([2, 4, 2, 6]));
    /// ```
    #[inline]
    pub fn maximum(self, other: Self) -> Self
    where
        B: PartialOrd,
    {
        let m = self.lt(other);
        self.blend(other, m)
    }

    /// A lane-wise maximum.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use slipstream::prelude::*;
    /// let a = u32x4::new([1, 4, 2, 5]);
    /// let b = u32x4::new([2, 3, 2, 6]);
    /// assert_eq!(a.minimum(b), u32x4::new([1, 3, 2, 5]));
    /// ```
    #[inline]
    pub fn minimum(self, other: Self) -> Self
    where
        B: PartialOrd,
    {
        let m = self.gt(other);
        self.blend(other, m)
    }

    // TODO: Example
    /// Sums the lanes together.
    ///
    /// The additions are done in a tree manner: `(a[0] + a[1]) + (a[2] + a[3])`.
    ///
    /// Note that this is potentially a slow operation. Prefer to do as many operations on whole
    /// vectors and only at the very end perform the horizontal operation.
    #[inline]
    pub fn horizontal_sum(self) -> B
    where
        B: Add<Output = B>,
    {
        #[inline(always)]
        fn inner<B: Copy + Add<Output = B>>(d: &[B]) -> B {
            if d.len() == 1 {
                d[0]
            } else {
                let mid = d.len() / 2;
                inner(&d[..mid]) + inner(&d[mid..])
            }
        }
        inner(&self.data)
    }

    /// Multiplies all the lanes of the vector.
    ///
    /// The multiplications are done in a tree manner: `(a[0] * a[1]) * (a[2] * a[3])`.
    ///
    /// Note that this is potentially a slow operation. Prefer to do as many operations on whole
    /// vectors and only at the very end perform the horizontal operation.
    #[inline]
    pub fn horizontal_product(self) -> B
    where
        B: Mul<Output = B>,
    {
        #[inline(always)]
        fn inner<B: Copy + Mul<Output = B>>(d: &[B]) -> B {
            if d.len() == 1 {
                d[0]
            } else {
                let mid = d.len() / 2;
                inner(&d[..mid]) * inner(&d[mid..])
            }
        }
        inner(&self.data)
    }

    cmp_op!(
        /// Lane-wise `==`.
        PartialEq => eq;

        /// Lane-wise `<`.
        PartialOrd => lt;

        /// Lane-wise `>`.
        PartialOrd => gt;

        /// Lane-wise `<=`.
        PartialOrd => le;

        /// Lane-wise `>=`.
        PartialOrd => ge;
    );
}

impl<A, B, const S: usize> Vector<A, B, S>
where
    A: Align,
    B: Repr + Float,
{
    /// Fused multiply-add. Computes (self * a) + b with only one rounding
    /// error, yielding a more accurate result than an unfused multiply-add.
    ///
    /// Using mul_add can be more performant than an unfused multiply-add if the
    /// target architecture has a dedicated fma CPU instruction.
    #[inline]
    pub fn mul_add(self, a: Self, b: Self) -> Self {
        let mut result = Self::splat(B::zero());
        for ((res, &s), (&a, &b)) in result
            .data
            .iter_mut()
            .zip(self.data.iter())
            .zip(a.data.iter().zip(b.data.iter()))
        {
            *res = s.mul_add(a, b);
        }
        result
    }
}

impl<A: Align, B: Repr, const S: usize> Masked for Vector<A, B, S> {
    type Mask = Vector<A, B::Mask, S>;
}

impl<A: Align, B: Default + Repr, const S: usize> Default for Vector<A, B, S> {
    #[inline]
    fn default() -> Self {
        Self::splat(Default::default())
    }
}

impl<A: Align, B: Debug + Repr, const S: usize> Debug for Vector<A, B, S> {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        fmt.debug_tuple("Vector").field(&self.data).finish()
    }
}

impl<A: Align, B: Repr, const S: usize> Deref for Vector<A, B, S> {
    type Target = [B; S];
    #[inline]
    fn deref(&self) -> &[B; S] {
        &self.data
    }
}

impl<A: Align, B: Repr, const S: usize> DerefMut for Vector<A, B, S> {
    #[inline]
    fn deref_mut(&mut self) -> &mut [B; S] {
        &mut self.data
    }
}

impl<A: Align, B: Repr, const S: usize> AsRef<[B]> for Vector<A, B, S> {
    #[inline]
    fn as_ref(&self) -> &[B] {
        &self.data
    }
}

impl<A: Align, B: Repr, const S: usize> AsRef<[B; S]> for Vector<A, B, S> {
    #[inline]
    fn as_ref(&self) -> &[B; S] {
        &self.data
    }
}

impl<A: Align, B: Repr, const S: usize> AsMut<[B]> for Vector<A, B, S> {
    #[inline]
    fn as_mut(&mut self) -> &mut [B] {
        &mut self.data
    }
}

impl<A: Align, B: Repr, const S: usize> AsMut<[B; S]> for Vector<A, B, S> {
    #[inline]
    fn as_mut(&mut self) -> &mut [B; S] {
        &mut self.data
    }
}

impl<A: Align, B: Repr, const S: usize> From<[B; S]> for Vector<A, B, S> {
    #[inline]
    fn from(data: [B; S]) -> Self {
        Self::assert_size();
        Self { _align: [], data }
    }
}

impl<A: Align, B: Repr, const S: usize> From<Vector<A, B, S>> for [B; S] {
    #[inline]
    fn from(vector: Vector<A, B, S>) -> [B; S] {
        vector.data
    }
}

impl<I, A, B, const S: usize> Index<I> for Vector<A, B, S>
where
    A: Align,
    B: Repr,
    [B; S]: Index<I>,
{
    type Output = <[B; S] as Index<I>>::Output;
    #[inline]
    fn index(&self, idx: I) -> &Self::Output {
        &self.data[idx]
    }
}

impl<I, A, B, const S: usize> IndexMut<I> for Vector<A, B, S>
where
    A: Align,
    B: Repr,
    [B; S]: IndexMut<I>,
{
    #[inline]
    fn index_mut(&mut self, idx: I) -> &mut Self::Output {
        &mut self.data[idx]
    }
}

impl<A: Align, B: AddAssign + Default + Repr, const S: usize> Sum for Vector<A, B, S> {
    #[inline]
    fn sum<I>(iter: I) -> Self
    where
        I: Iterator<Item = Self>,
    {
        let mut result = Self::default();
        for i in iter {
            result += i;
        }

        result
    }
}

impl<A: Align, B: MulAssign + Repr, const S: usize> Product for Vector<A, B, S> {
    #[inline]
    fn product<I>(iter: I) -> Self
    where
        I: Iterator<Item = Self>,
    {
        let mut result = Self::splat(B::ONE);
        for i in iter {
            result *= i;
        }

        result
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
bin_op_impl!(Shl, shl, ShlAssign, shl_assign);
bin_op_impl!(Shr, shr, ShrAssign, shr_assign);

una_op_impl!(Neg, neg);
una_op_impl!(Not, not);

impl<A: Align, B: PartialEq + Repr, const S: usize> PartialEq for Vector<A, B, S> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl<A: Align, B: Eq + Repr, const S: usize> Eq for Vector<A, B, S> {}

impl<A: Align, B: PartialEq + Repr, const S: usize> PartialEq<[B; S]> for Vector<A, B, S> {
    #[inline]
    fn eq(&self, other: &[B; S]) -> bool {
        self.data == *other
    }
}

impl<A: Align, B: PartialEq + Repr, const S: usize> PartialEq<Vector<A, B, S>> for [B; S] {
    #[inline]
    fn eq(&self, other: &Vector<A, B, S>) -> bool {
        *self == other.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;

    type V = u16x4;

    #[test]
    #[should_panic(expected = "Creating vector from the wrong sized slice (expected 4, got 3)")]
    fn wrong_size_new() {
        V::new([1, 2, 3]);
    }

    #[test]
    fn round_trip() {
        let orig = [1, 2, 3, 4];
        assert_eq!(<[u16; 4]>::from(u16x4::from(orig)), orig);
    }

    #[test]
    fn shuffle() {
        let v1 = V::new([1, 2, 3, 4]);
        let v2 = V::gather_load(v1, [3, 1, 2, 0]);
        assert_eq!(v2.deref(), &[4, 2, 3, 1]);
        let v3 = V::gather_load(v2, [0, 0, 2, 2]);
        assert_eq!(v3.deref(), &[4, 4, 3, 3]);
    }

    #[test]
    fn gather() {
        let data = (1..=10).collect::<Vec<_>>();
        let v = V::gather_load(data, [0, 2, 4, 6]);
        assert_eq!(v, [1, 3, 5, 7]);
    }

    #[test]
    fn scatter() {
        let v = V::new([1, 2, 3, 4]);
        let mut output = [0; 10];
        v.scatter_store(&mut output, [1, 3, 5, 7]);
        assert_eq!(output, [0, 1, 0, 2, 0, 3, 0, 4, 0, 0]);
    }

    #[test]
    #[should_panic(expected = "Gather out of bounds")]
    fn gather_oob() {
        V::gather_load([1, 2, 3], [0, 1, 2, 3]);
    }

    #[test]
    #[should_panic(expected = "Gathering vector from wrong number of indexes")]
    fn gather_idx_cnt() {
        V::gather_load([0, 1, 2, 3, 4], [0, 1]);
    }

    #[test]
    #[should_panic(expected = "Scatter out of bounds")]
    fn scatter_oob() {
        let mut out = [0; 10];
        V::new([1, 2, 3, 4]).scatter_store(&mut out, [0, 1, 2, 15]);
    }

    #[test]
    #[should_panic(expected = "Scattering vector to wrong number of indexes")]
    fn scatter_idx_cnt() {
        let mut out = [0; 10];
        V::new([1, 2, 3, 4]).scatter_store(&mut out, [0, 1, 2]);
    }

    // TODO: Tests for out of bounds index on masked loads/stores + tests for index out of bound
    // but disabled by the mask

    const T: m32 = m32::TRUE;
    const F: m32 = m32::FALSE;

    #[test]
    fn cmp() {
        let v1 = u32x4::new([1, 3, 5, 7]);
        let v2 = u32x4::new([2, 3, 4, 5]);

        assert_eq!(v1.eq(v2), m32x4::new([F, T, F, F]));
        assert_eq!(v1.le(v2), m32x4::new([T, T, F, F]));
        assert_eq!(v1.ge(v2), m32x4::new([F, T, T, T]));
    }

    #[test]
    fn blend() {
        let v1 = u32x4::new([1, 2, 3, 4]);
        let v2 = u32x4::new([5, 6, 7, 8]);

        let b1 = v1.blend(v2, m32x4::new([F, T, F, T]));
        assert_eq!(b1, u32x4::new([1, 6, 3, 8]));

        let b2 = v1.blend(v2, [false, true, false, true]);
        assert_eq!(b1, b2);
    }

    #[test]
    fn fma() {
        let a = f32x4::new([1.0, 2.0, 3.0, 4.0]);
        let b = f32x4::new([5.0, 6.0, 7.0, 8.0]);
        let c = f32x4::new([9.0, 10.0, 11.0, 12.0]);

        assert_eq!(a.mul_add(b, c), f32x4::new([14.0, 22.0, 32.0, 44.0]));
    }
}
