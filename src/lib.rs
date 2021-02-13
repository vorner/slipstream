#![doc(test(attr(deny(warnings))))]
#![warn(missing_docs)]
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
//! environment. However, the optimisations are not guaranteed. In particular, while the crate may
//! allow for a significant speed-ups, it can *also make your code slower*. When using the crate,
//! you're strongly advised to benchmark.
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
//! All these types are backed by the generic [`Vector`] type.  See the
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
//! ```rust
//! # use slipstream::prelude::*;
//! fn sum(data: &[f32x8]) -> f32 {
//!     data
//!         .iter()
//!         .copied()
//!         .sum::<f32x8>() // Summing up whole f32x8 vectors, result is also f32x8
//!         .horizontal_sum() // Summing individual lanes of that vector
//! }
//! # assert_eq!(0.0, sum(&[]));
//! ```
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
//! [`multiversion`]: https://crates.io/crates/multiversion
//! [`rayon`]: https://crates.io/crates/rayon
//! [`packed_simd`]: https://crates.io/crates/packed_simd
//! [`faster`]: https://crates.io/crates/faster
//! [`simdeez`]: https://crates.io/crates/simdeez
//! [`safe_simd`]: https://github.com/calebzulawski/safe_simd/

pub mod iterators;
pub mod mask;
pub mod types;
pub mod vector;

pub use iterators::Vectorizable;
pub use mask::Mask;
pub use types::*;
pub use vector::Vector;

/// Commonly used imports
///
/// This can be imported to get all the vector types and all the relevant user-facing traits of the
/// crate.
pub mod prelude {
    pub use crate::types::*;
    pub use crate::vector::Masked as _;
    pub use crate::Mask as _;
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

/// Free-standing version of [`Vectorizable::vectorize`].
///
/// This is the same as `a.vectorize()`. Nevertheless, this version might be more convenient as it
/// allows hinting the result vector type with turbofish.
///
/// ```rust
/// # use slipstream::prelude::*;
/// let data = [1, 2, 3, 4];
/// for v in slipstream::vectorize::<u32x2, _>(&data[..]) {
///     println!("{:?}", v);
/// }
/// ```
#[inline(always)]
pub fn vectorize<V, A>(a: A) -> impl Iterator<Item = V>
where
    A: Vectorizable<V>,
{
    a.vectorize()
}

/// Free-standing version of [`Vectorizable::vectorize_pad`].
///
/// Equivalent to `a.vectorize_pad(pad)`, but may be more convenient or readable in certain cases.
///
/// ```rust
/// # use slipstream::prelude::*;
/// let data = [1, 2, 3, 4, 5, 6];
/// let v = slipstream::vectorize_pad(&data[..], i32x4::splat(-1)).collect::<Vec<_>>();
/// assert_eq!(v, vec![i32x4::new([1, 2, 3, 4]), i32x4::new([5, 6, -1, -1])]);
/// ```
#[inline(always)]
pub fn vectorize_pad<V, A>(a: A, pad: A::Padding) -> impl Iterator<Item = V>
where
    A: Vectorizable<V>,
{
    a.vectorize_pad(pad)
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
