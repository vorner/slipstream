//! Bool-like types used for masked operations.
//!
//! With multi-lane vectors, it is sometimes useful to do a lane-wise comparison or to disable some
//! of the lanes for a given operation. Naturally, one would express this using a correctly sized
//! `bool` array.
//!
//! Nevertheless, the CPU SIMD instructions don't use bools, but signal `true`/`false` with a
//! full-sized type with either all bits set to 1 or 0 (TODO: this is not true for AVX-512, what do
//! we want to do about it?). Therefore, we define our own types that act like bools, but are
//! represented in the above way. The comparison operators return vectors of these base mask types.
//! The selection operations accept whatever mask vector with the same number of lanes, but they
//! are expected to act fastest with the correct sized ones.
//!
//! For the purpose of input, `bool` is also considered a mask type.
//!
//! The interesting operations are:
//! * Comparisons ([`lt`][crate::Vector::lt], [`le`][crate::Vector::le], [`eq`][crate::Vector::eq],
//!   [`ge`][crate::Vector::ge], [`gt`][crate::Vector::gt])
//! * The [`blend`][crate::Vector::blend] method.
//! * Masked [loading][crate::Vector::gather_load_masked] and
//!   [storing][crate::Vector::scatter_store_masked] of vectors.
//!
//! The number in the type name specifies the number of bits. Therefore, for the
//! [`u16x4`][crate::u16x4], the natural mask type is a vector of 4 [`m16`], which is
//! [`m16x4`][crate::m16x4].
//!
//! While it is possible to operate with the bools (by converting them), it is more common to
//! simply pipe the masks back into the vectors. Note that they *do* implement the usual boolean
//! operators (however, only the non-shortcircuiting/bitwise variants). These work lane-wise.
//!
//! # Examples
//!
//! ```rust
//! # use slipstream::prelude::*;
//! fn abs(vals: &mut [i32]) {
//!     let zeroes = i32x8::default();
//!     for mut v in vals.vectorize_pad(i32x8::default()) {
//!         // Type of this one is m32x8 and is true whereever the lane isnegative.
//!         let negative = v.lt(zeroes);
//!         // Pick lanes from v where non-negative, pick from -v where negative.
//!         *v = v.blend(-*v, negative);
//!     }
//! }
//! let mut data = [1, -2, 3];
//! abs(&mut data);
//! assert_eq!(data, [1, 2, 3]);
//! ```
use core::ops::*;

mod inner {
    pub trait Sealed {}
}

/// The trait implemented by all the mask types.
///
/// Note that this trait is not implementable by downstream crates, as code in the crate assumes
/// (and relies for safety on the assumption) that the type can ever hold only the two values.
///
/// See the [module documentation][crate::mask].
pub trait Mask:
    Copy
    + Eq
    + Send
    + Sync
    + inner::Sealed
    + Not
    + BitAnd
    + BitAndAssign
    + BitOr
    + BitOrAssign
    + BitXor
    + BitXorAssign
    + 'static
{
    /// A constant specifying the true value of the type.
    ///
    /// For bool, this is `true`. For the others, this means all bits set to `1` ‒ eg. 256 for
    /// [`m8].
    const TRUE: Self;

    /// The false value of the type.
    ///
    /// For bool, this is `false`. For the others, this means 0 (all bits set to 0).
    const FALSE: Self;

    /// Converts the type to bool.
    #[inline]
    fn bool(self) -> bool {
        if self == Self::TRUE {
            true
        } else if self == Self::FALSE {
            false
        } else {
            unsafe { core::hint::unreachable_unchecked() }
        }
    }

    /// Converts the type from bool.
    #[inline]
    fn from_bool(v: bool) -> Self {
        if v {
            Self::TRUE
        } else {
            Self::FALSE
        }
    }
}

/// Inner implementation of the mask types.
///
/// This is to be used through the type aliases in this module, like [`m8`], or more often through
/// vectors of these, like [`m8x4`][crate::m8x4]. These are the [`mask vectors`][crate::mask].
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub struct MaskWrapper<I>(I);

macro_rules! trait_impl {
    ($T: ident, $m: ident, $TA: ident, $ma: ident) => {
        impl<I: $T<Output = I>> $T for MaskWrapper<I> {
            type Output = Self;
            fn $m(self, rhs: Self) -> Self {
                Self((self.0).$m(rhs.0))
            }
        }

        impl<I: $TA> $TA for MaskWrapper<I> {
            fn $ma(&mut self, rhs: Self) {
                (self.0).$ma(rhs.0)
            }
        }
    };
}

trait_impl!(BitAnd, bitand, BitAndAssign, bitand_assign);
trait_impl!(BitOr, bitor, BitOrAssign, bitor_assign);
trait_impl!(BitXor, bitxor, BitXorAssign, bitxor_assign);

impl<I: Not<Output = I>> Not for MaskWrapper<I> {
    type Output = Self;
    fn not(self) -> Self::Output {
        Self(self.0.not())
    }
}

#[allow(missing_docs)]
pub type m8 = MaskWrapper<u8>;

impl inner::Sealed for m8 {}

impl Mask for m8 {
    const TRUE: Self = MaskWrapper(u8::MAX);
    const FALSE: Self = MaskWrapper(0);
}

#[allow(missing_docs)]
pub type m16 = MaskWrapper<u16>;

impl inner::Sealed for m16 {}

impl Mask for m16 {
    const TRUE: Self = MaskWrapper(u16::MAX);
    const FALSE: Self = MaskWrapper(0);
}

#[allow(missing_docs)]
pub type m32 = MaskWrapper<u32>;

impl inner::Sealed for m32 {}

impl Mask for m32 {
    const TRUE: Self = MaskWrapper(u32::MAX);
    const FALSE: Self = MaskWrapper(0);
}

#[allow(missing_docs)]
pub type m64 = MaskWrapper<u64>;

impl inner::Sealed for m64 {}

impl Mask for m64 {
    const TRUE: Self = MaskWrapper(u64::MAX);
    const FALSE: Self = MaskWrapper(0);
}

#[allow(missing_docs)]
pub type m128 = MaskWrapper<u128>;

impl inner::Sealed for m128 {}

impl Mask for m128 {
    const TRUE: Self = MaskWrapper(u128::MAX);
    const FALSE: Self = MaskWrapper(0);
}

#[allow(missing_docs)]
pub type msize = MaskWrapper<usize>;

impl inner::Sealed for msize {}

impl Mask for msize {
    const TRUE: Self = MaskWrapper(usize::MAX);
    const FALSE: Self = MaskWrapper(0);
}

impl inner::Sealed for bool {}

impl Mask for bool {
    const TRUE: Self = true;
    const FALSE: Self = false;
}
