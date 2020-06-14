#![doc(
    html_root_url = "https://docs.rs/slipstream/0.1.0/slipstream/",
    test(attr(deny(warnings))),
)]
// TODO: Enable this? #![deny(missing_docs, warnings)]
#![allow(non_camel_case_types)]
#![cfg_attr(not(test), no_std)]

//! This library helps writing code in a way that incentives the compiler to
//! optimize the results better (without really doing anything itself).
//!
//! Modern compilers, including `rustc`, are able to come up with impressive ways to
//! speed up the resulting code, using techniques like loop unrolling and
//! autovectorization, routinely outperforming what one would hand-craft.
//! Nevertheless, each optimisation has some assumptions that must be proven to hold
//! before it can be applied.
//!
//! This library offers „vector“ types, like [`u16x8`], which act in a very similar
//! way as little fixed-sized arrays (in this case it would be `[u16; 8]`), but with
//! arithmetics defined for them. They also enforce alignment of the whole vectors.
//! Therefore, one can write the algorithm in a way that works on these groups of
//! data and make it easier for the compiler to prove the assumptions. This can
//! result in multiple factor speed ups by giving the compiler these proofs „for
//! free“ and allowing it to apply aggressive optimizations.
//!
//! Unlike several other SIMD libraries, this one doesn't do any actual explicit SIMD. That results
//! in relatively simpler interface while still working on stable compiler. It also works in no-std
//! environment.
//!
//! # Anatomy of the crate
//!
//! ## Vector types
//!
//! On the surface, there are types like [`u16x8`], which is just an wrapper around `[u16; 8]`.
//! These wrappers act a bit like arrays (they can be dereferenced to a slice, they can be indexed)
//! and have **common arithmetic traits** implemented. The arithmetic is applied to each index
//! separately, eg:
//!
//! ```
//! # use slipstream::prelude::*;
//! let a = u8x2::new([1, 2]);
//! let b = u8x2::new([3, 4]);
//! assert_eq!(a + b, u8x2::new([4, 6]));
//! ```
//!
//! Most of their methods are provided by traits, **especially the [`Vector`] trait**. See the
//! methods there to see how they can be created and how they interact.
//!
//! All these can be imported by importing prelude:
//!
//! ```
//! # #[allow(unused_imports)]
//! use slipstream::prelude::*;
//! ```
//!
//! The names are based on primitive types, therefore there are types like [`u8x2`], [`i8x2`],
//! [`f32x4`], [`f64x2`].
//!
//! There are some more types:
//!
//! * [`wu8x2`] is based on [`Wrapping<u8>`][core::num::Wrapping], [`wi8x2`] is based on
//!   [`Wrapping<i8>`][core::num::Wrapping].
//! * [`bx2`] are vectors of [`bool`]s.
//! * [`m8x2`] are mask vectors. They act *a bit* like booleans, but they have width and use all
//!   bits set to `1` for `true`. These can be used to [`blend`][Vector::blend] vectors together,
//!   mask loads and stores and are results of comparisons. The representation is inspired by what
//!   the vector instructions actually use, so they should be possible for the compiler to
//!   autovectorize. The widths match the types they work with ‒ comparing two [`u32x2`]s will
//!   result in [`m32x2`]. The lanes can be converted to/from [`bool`] with methods on the [`Mask`]
//!   trait, but usually these are just fed back to some other vector operations.
//!
//! ## Under the hood
//!
//! The above types are only type aliases. Under the hood, these are implemented in generic way by
//! types like [`Packed4`]. These can be parametrized by a type and a
//! length. While the crate disallows carrying different base types (for safety reasons; the crate
//! makes certain assumptions about the base type), different array lengths are possible ‒ though
//! for the compiler to make any sense of them, likely only power of two lengths make any sense.
//! The lengths are specified using the type level numbers from the [`typenum`] crate, eg.
//! [`U4`][typenum::consts::U4].
//!
//! The suffix number (4 in [`Packed4`]) means minimal alignment of the whole type, in bytes.
//! Therefore, while [`u8`] aligns to 1 byte, `[u8; 16]` also aligns to 1 byte, [`u8x16`] aligns to
//! 16 bytes (because it uses the [`Packed16`] type). This allows the compiler to use 128bit vector
//! instructions (SSE) as these expect such alignment.
//!
//! ## Vectorization of slices
//!
//! While it might be better for performance to store all data already in the vector types, it
//! oftentimes happen that the input is in form of a slice or multiple slices of the primitive
//! types. It would be possible to chunk the input and load them into the vectors one at a time,
//! either manually or by using something like the [`chunks_exact`][core::slice::ChunksExact]
//! and [`zip`][core::iter::Iterator::zip]. Nevertheless, it turns out to be inconvenient and often
//! too complex for the compiler to make sense of and vectorize properly.
//!
//! Therefore, the crate provides its own means for splitting the data into vectors, using the
//! [`Vectorizable`] trait. This is implemented on const and mutable slices as well as tuples and
//! small (fixed-sized) arrays of these. The trait adds the [`vectorize`][Vectorizable::vectorize]
//! and [`vectorize_pad`][Vectorizable::vectorize_pad] methods.
//!
//! As the methods can't know into how wide vectors the input should be split, it is often needed
//! to provide a type hint somewhere.
//!
//! ```rust
//! # use slipstream::prelude::*;
//! fn dot_product(l: &[f32], r: &[f32]) -> f32 {
//!     let mut result = f32x8::default();
//!     // This assumes l and r are of the same length and divisible by 8
//!     for (l, r) in (l, r).vectorize() {
//!         // Force the exact type of l and r vectors
//!         let (l, r): (f32x8, f32x8) = (l, r);
//!         result += l * r;
//!     }
//!     // Sum the 8 lanes together
//!     result.horizontal_sum()
//! }
//! # dot_product(&[], &[]);
//! ```
//!
//! # Multiversioning and dynamic instruction set selection
//!
//! If used as in the examples above, the compiler chooses an instruction set at compile time,
//! based on the command line arguments. By default these are conservative, to run on arbitrary
//! (old) CPU. It is possible to either enable newer instructions at compile time (at the cost of
//! not being able to run the program on the older CPUs) or compile multiple versions of the same
//! function and choose the right one at runtime, depending on what the CPU actually supports.
//!
//! While this library doesn't provide any direct support for multiversioning, it has been observed
//! to work reasonably well in combination with the [`multiversion`] crate.
//!
//! Note that using a newer and richer instruction set is not always a win. In some cases it can
//! even lead to performance degradation. In particular:
//!
//! * Wide enough vectors must be used to take advantage of the 256 or more bits of the newer
//!   instruction set (using these with older instruction set is not a problem; the vector
//!   operations will simply translate to multiple narrower instructions). This might create larger
//!   „leftovers“ on the ends of slices that need to be handled in non-vectorized manner.
//! * The CPU may need to switch state, possibly negotiate a higher power supply. This might lead
//!   to slow down before that happens and might degrade performance of neighboring cores.
//! * Some AMD processors (Buldozers) know the instructions, but simulate them by dispatching the
//!   narrower instructions internally (at least it seems so, one 256bit instruction takes a bit
//!   longer than two 128bit ones).
//!
//! Depending on the workload, both slowdowns and full 2* speedups were observed. The chances of
//! speedups are higher when there's a lot of data to crunch „in one go“ (so the CPU has time to
//! „warm up“, the leftovers don't matter that much, etc).
//!
//! # Performance tuning tips
//!
//! The sole purpose of this library is to get faster programs, so here are few things to keep in
//! mind when trying.
//!
//! This library (or SIMD in general) is not a silver bullet. It's good to tackle a lot of data
//! crunching by sheer force (the hammer style approach), but can yield only multiplicative
//! speedups (depending on the width of the instructions, on the size of the base type, etc, one
//! can't expect more than 10 or 20 times speedup, usually less). Oftentimes, more high level
//! optimizations bring significantly better results ‒ choosing a better algorithm, reordering the
//! data in memory to avoid cache misses. These can give you orders of magnitude in some cases.
//! Also, besides instruction level parallelism, one can try using threads to parallelize across
//! cores (for example using [`rayon`]). Therefore, vectorization should be used in the latter
//! stages of performance tuning.
//!
//! Also note that when used on a platform without any SIMD support, it can lead to both speed ups
//! (due to loop unrolling) and slowdowns (probably due to exhaustion of available CPU registers).
//!
//! It is important to measure and profile. Not only because you want to spend the time optimizing
//! the hot parts of the program which actually take significant amount of time, but because the
//! autovectorizer and compiler optimizations sometimes produce surprising results.
//!
//! ## Performance characteristics
//!
//! In general, simple lane-wise operations are significantly faster than horizontal operations
//! (when neighboring lanes may interact) and complex ones. Therefore, adding two vectors using the
//! `+` operator is likely to end up being faster than the
//! [`horizontal_sum`][Vector::horizontal_sum] or the [`gather_load`][Vector::gather_load]
//! constructor.
//!
//! It is advisable to keep as much in vectors as possible instead of operating on separate lanes.
//!
//! Therefore, to compute a sum of bunch of numbers, split the input into vectors, sum these up and
//! do single `horizontal_sum` at the very end.
//!
//! Also keep in mind that there's usually some „warm up“ for vectorized part of code. This partly
//! comes from the need to somehow deal with uneven ends (if the input is not divisible by the
//! vector size). Also, some instructions require the CPU to switch state, possibly lower frequency
//! and negotiate higher power supply, which may even hinder performance of neighboring cores (this
//! is more of a problem for „newer“ instruction sets like AVX-512 than eg. SSE).
//!
//! Therefore, there's little advantage of interspersing otherwise non-vectorized code with
//! occasional vector variable. The best results are for crunching big inputs all at once.
//!
//! ## Suggested process
//!
//! * Write the non-vectorized version first. Make sure to use the correct algorithm, avoid
//!   unnecessary work, etc.
//! * Parallelize it across threads where it makes sense.
//! * Prepare a micro-benchmark exercising the hot part.
//! * Try rewriting it using the vector types in this crate, but keep the non-vectorized version
//!   around for comparison. Make sure to run the benchmark for both.
//! * If the vectorized version doesn't meet the expectations (or even make things slower), you can
//!   check these things:
//!   - If using the [`multiversion`] crate, watch out for (not) inlining. The detected instruction
//!     set is not propagated to other functions called from the multiversioned one, only to the
//!     inlined ones.
//!   - Make sure to use reasonably sized vector type. On one side, it needs to be large enough to
//!     fill the whole SIMD register (128 bit for SSE and NEON, 256 for AVX, 512 bits for AVX-512).
//!     On the other side, it should not be too large ‒ while wider vectors can be simulated by
//!     executing multiple narrower instructions, they also take multiple registers and that may
//!     lead to unnecessary „juggling“.
//!   - See the profiler output if any particular part stands out. Oftentimes, some constructs like
//!     the [`zip`][core::iter::Iterator::zip] iterator adaptor were found to be problematic. If a
//!     construct is too complex for rustc to „see through“, it can be helped by rewriting that
//!     particular part manually in a simpler way. Pulling slice range checks before the loop might
//!     help too, as rustc no longer has to ensure a panic from the violation would happen at the
//!     right time in the middle of processing.
//!   - Check the assembler output if it looks sane. Seeing if it looks vectorized can be done
//!     without extensive assembler knowledge ‒ SIMD instructions have longer names and use
//!     different named registers (`xmm?` ones for SSE, `ymm?` ones for AVX).
//!
//! See if the profiler can be configured to show inlined functions instead of counting the whole
//! runtime to the whole function. Some profilers can even show annotated assembler code,
//! pinpointing the instruction or area that takes long time. In such case, be aware that an
//! instruction might take a long time because it waits on a data dependency (some preceding
//! instruction still being executed in the pipeline) or data from memory.
//!
//! For the `perf` profile, this can be done with `perf record --call-graph=dwarf <executable>`,
//! `perf report` and `perf annotate`. Make sure to profile with both optimizations *and* debug
//! symbols enabled (but if developing a proprietary thing, make sure to ship *without* the debug
//! symbols).
//!
//! ```toml
//! [profile.release]
//! debug = 2
//! ```
//!
//! When all else fails, you can always rewrite only parts of the algorithm using the explicit
//! intrinsics in [`core::arch`] and leave the rest for autovectorizer. The vector types should be
//! compatible for transmuting to the low-level vectors (eg. `__m128`).
//!
//! # Alternatives
//!
//! There are other crates that try to help with SIMD:
//!
//! * [`packed_simd`]: This is *the* official SIMD library. The downside is, this works only on
//!   nighty compiler and the timeline when this could get stabilized is unclear.
//! * [`faster`]: Works only on nightly and looks abandoned.
//! * [`simdeez`]: Doesn't have unsigned ints. Works on stable, but is unsound (can lead to UB
//!   without writing a single line of user `unsafe` code).
//! * [`safe_simd`]: It has somewhat more complex API than this library, because it deals with
//!   instruction sets explicitly. It supports explicit vectorization (doesn't rely on
//!   autovectorizer). It is not yet released.
//!
//! [`Packed4`]: crate::vector::Packed4
//! [`Packed16`]: crate::vector::Packed16
//! [`multiversion`]: https://crates.io/crates/multiversion
//! [`rayon`]: https://crates.io/crates/rayon
//! [`packed_simd`]: https://crates.io/crates/packed_simd
//! [`faster`]: https://crates.io/crates/faster
//! [`simdeez`]: https://crates.io/crates/simdeez
//! [`safe_simd`]: https://github.com/calebzulawski/safe_simd/

use core::ops::*;

use generic_array::ArrayLength;
use typenum::marker_traits::Unsigned;

pub mod iterators;
pub mod mask;
pub mod types;
pub mod vector;

pub use iterators::Vectorizable;
pub use mask::Mask;
pub use types::*;

/// Commonly used imports
///
/// This can be imported to get all the vector types and all the relevant user-facing traits of the
/// crate.
pub mod prelude {
    pub use crate::types::*;
    pub use crate::Mask as _;
    pub use crate::Vector as _;
    pub use crate::Vectorizable as _;
}

mod inner {
    use core::num::Wrapping;

    use crate::mask::{m128, m16, m32, m64, m8, msize, Mask};

    /// A trait to enable vectors to use this type as the base type.
    ///
    /// This is in a private module to prevent users creating their own „crazy“ vector
    /// implementations. We make some non-trivial assumptions about the inner types and be are
    /// conservative at least until we figure out what *exact* assumptions these are and formalize
    /// them.
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

/// A trait with common methods of the vector types.
///
/// The vector types (like [`u32x4`]) don't have inherent methods on themselves. They implement
/// several traits (mostly arithmetics, bit operations, dereferencing to slices and indexing).
/// Further methods of all the vector types are on this trait.
///
/// It can also be used to describe multiple vector types at once ‒ for example `Vector<Base =
/// u32>` describes all the vectors that have `u32` as their base type, be it [`u32x4`] or
/// [`u32x16`].
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
pub trait Vector: Copy + Send + Sync + Sized + 'static {
    /// Type of one lane of the vector.
    ///
    /// It's the `u32` for [`u32x4`].
    type Base: inner::Repr;

    /// A type-level integer specifying the length of the vector.
    ///
    /// This is the [`U4`][typenum::consts::U4] for [`u32x4`].
    type Lanes: ArrayLength<Self::Base>;

    /// The mask type for this vector.
    ///
    /// Masks are vector types of boolean-like base types. They are used as results of lane-wise
    /// comparisons like [`eq`][Vector::eq] and for enabling subsets of lanes for certain
    /// operations, like [`blend`][Vector::blend] and
    /// [`gather_load_masked`][Vector::gather_load_masked].
    ///
    /// This associated types describes the native mask for the given vector. For example for
    /// [`u32x4`] it would be [`m32x4`]. This is the type that the comparisons produce. While the
    /// selection methods accept any mask type of the right number of lanes, using this type on
    /// their input is expected to yield the best performance.
    type Mask: AsRef<[<Self::Base as inner::Repr>::Mask]>;

    /// Number of lanes of the vector.
    ///
    /// This is similar to [`Lanes`][Vector::Lanes], but as a constant instead of type.
    const LANES: usize = Self::Lanes::USIZE;

    /// Load the vector without doing bounds checks.
    ///
    /// # Safety
    ///
    /// The pointed to memory must be valid in `Self::LANES` consecutive cells ‒ eg. it must
    /// contain a full array of the base types.
    unsafe fn new_unchecked(input: *const Self::Base) -> Self;

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
    fn new<I>(input: I) -> Self
    where
        I: AsRef<[Self::Base]>,
    {
        let input = input.as_ref();
        assert_eq!(
            input.len(),
            Self::LANES,
            "Creating vector from the wrong sized slice (expected {}, got {})",
            Self::LANES,
            input.len(),
        );
        unsafe { Self::new_unchecked(input.as_ptr()) }
    }

    /// Produces a vector of all lanes set to the same value.
    ///
    /// ```rust
    /// # use slipstream::prelude::*;
    /// let v = f32x4::splat(1.2);
    /// assert_eq!(v, f32x4::new([1.2, 1.2, 1.2, 1.2]));
    /// ```
    fn splat(value: Self::Base) -> Self;

    /// Loads the vector from a slice by indexing it.
    ///
    /// Unlike the [`new`], this can load the vector from discontinuous parts of the slice, out of
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
    fn gather_load<I, Idx>(input: I, idx: Idx) -> Self
    where
        I: AsRef<[Self::Base]>,
        Idx: AsRef<[usize]>;

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
    fn gather_load_masked<I, Idx, M, MB>(self, input: I, idx: Idx, mask: M) -> Self
    where
        I: AsRef<[Self::Base]>,
        Idx: AsRef<[usize]>,
        M: AsRef<[MB]>,
        MB: Mask;

    /// Store the vector into a slice by indexing it.
    ///
    /// This is the inverse of [`gather_load`][Vector::gather_load]. It takes the lanes of the
    /// vector and stores them into the slice into given indices.
    ///
    /// If you want to store it into a continuous slice, it is potentially faster to do it using
    /// the `copy_from_slice` method:
    ///
    /// ```rust
    /// # use slipstream::prelude::*;
    /// let mut data = vec![0; 6];
    /// let v = u32x4::new([1, 2, 3, 4]);
    /// data[0..4].copy_from_slice(&v);
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
    fn scatter_store<O, Idx>(self, output: O, idx: Idx)
    where
        O: AsMut<[Self::Base]>,
        Idx: AsRef<[usize]>;

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
    fn scatter_store_masked<O, Idx, M, MB>(self, output: O, idx: Idx, mask: M)
    where
        O: AsMut<[Self::Base]>,
        Idx: AsRef<[usize]>,
        M: AsRef<[MB]>,
        MB: Mask;

    /// Lane-wise `<`.
    fn lt(self, other: Self) -> Self::Mask
    where
        Self::Base: PartialOrd;

    /// Lane-wise `>`.
    fn gt(self, other: Self) -> Self::Mask
    where
        Self::Base: PartialOrd;

    /// Lane-wise `<=`.
    fn le(self, other: Self) -> Self::Mask
    where
        Self::Base: PartialOrd;

    /// Lane-wise `>=`.
    fn ge(self, other: Self) -> Self::Mask
    where
        Self::Base: PartialOrd;

    /// Lane-wise `==`.
    fn eq(self, other: Self) -> Self::Mask
    where
        Self::Base: PartialEq;

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
    fn blend<M, MB>(self, other: Self, mask: M) -> Self
    where
        M: AsRef<[MB]>,
        MB: Mask;

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
    fn maximum(self, other: Self) -> Self
    where
        Self::Base: PartialOrd,
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
    fn minimum(self, other: Self) -> Self
    where
        Self::Base: PartialOrd,
    {
        let m = self.gt(other);
        self.blend(other, m)
    }

    /// Sums the lanes together.
    ///
    /// The additions are done in a tree manner: `(a[0] + a[1]) + (a[2] + a[3])`.
    fn horizontal_sum(self) -> Self::Base
    where
        Self::Base: Add<Output = Self::Base>;


    /// Multiplies all the lanes of the vector.
    ///
    /// The multiplications are done in a tree manner: `(a[0] * a[1]) * (a[2] * a[3])`.
    fn horizontal_product(self) -> Self::Base
    where
        Self::Base: Mul<Output = Self::Base>;
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn minmax() {
        let a = u32x4::new([1, 4, 8, 9]);
        let b = u32x4::new([3, 3, 5, 11]);

        assert_eq!(a.minimum(b), u32x4::new([1, 3, 5, 9]));
        assert_eq!(a.maximum(b), u32x4::new([3, 4, 8, 11]));
        assert_eq!(a.minimum(b), b.minimum(a));
        assert_eq!(a.maximum(b), b.maximum(a));
        assert_eq!(a.maximum(b).ge(a.minimum(b)), m32x4::splat(m32::TRUE));
    }
}
