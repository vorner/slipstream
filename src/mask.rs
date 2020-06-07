use core::ops::*;

mod inner {
    pub trait Sealed {}
}

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
    const TRUE: Self;
    const FALSE: Self;

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

    #[inline]
    fn from_bool(v: bool) -> Self {
        if v {
            Self::TRUE
        } else {
            Self::FALSE
        }
    }
}

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

pub type m8 = MaskWrapper<u8>;

impl inner::Sealed for m8 {}

impl Mask for m8 {
    const TRUE: Self = MaskWrapper(u8::MAX);
    const FALSE: Self = MaskWrapper(0);
}

pub type m16 = MaskWrapper<u16>;

impl inner::Sealed for m16 {}

impl Mask for m16 {
    const TRUE: Self = MaskWrapper(u16::MAX);
    const FALSE: Self = MaskWrapper(0);
}

pub type m32 = MaskWrapper<u32>;

impl inner::Sealed for m32 {}

impl Mask for m32 {
    const TRUE: Self = MaskWrapper(u32::MAX);
    const FALSE: Self = MaskWrapper(0);
}

pub type m64 = MaskWrapper<u64>;

impl inner::Sealed for m64 {}

impl Mask for m64 {
    const TRUE: Self = MaskWrapper(u64::MAX);
    const FALSE: Self = MaskWrapper(0);
}

pub type m128 = MaskWrapper<u128>;

impl inner::Sealed for m128 {}

impl Mask for m128 {
    const TRUE: Self = MaskWrapper(u128::MAX);
    const FALSE: Self = MaskWrapper(0);
}

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
