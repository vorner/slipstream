#![cfg_attr(not(feature = "std"), no_std)]
#![allow(non_camel_case_types)]

use core::fmt::{Debug, Display, Formatter, Result as FmtResult};
use core::iter;
use core::marker::PhantomData;
use core::num::Wrapping;
use core::ops::*;
use core::slice;

use generic_array::{ArrayLength, GenericArray};
use typenum::consts::*;
use typenum::marker_traits::Unsigned;

mod inner {
    use core::num::Wrapping;

    use typenum::consts::*;
    use typenum::marker_traits::Unsigned;

    pub trait InstructionSet: Sized { }

    pub unsafe trait Repr<For>: Copy {
        type LANE_MULTIPLYIER: Unsigned;
    }

    unsafe impl Repr<u16> for Wrapping<u16> {
        type LANE_MULTIPLYIER = U1;
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct InstructionSetNotAvailable(pub &'static str);

impl Display for InstructionSetNotAvailable {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        write!(fmt, "Instruction set {} not available", self.0)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for InstructionSetNotAvailable {}

pub trait InstructionSet: Copy + Debug + inner::InstructionSet {
    fn detect() -> Result<Self, InstructionSetNotAvailable>;
    type u16x4: Vector<u16, U4, Self>;
    type u16x8: Vector<u16, U8, Self>;
}

// It's OK to let users create this one directly, it's safe to use always.
#[derive(Copy, Clone, Debug)]
struct Polyfill;

impl inner::InstructionSet for Polyfill { }

impl InstructionSet for Polyfill {
    #[inline]
    fn detect() -> Result<Self, InstructionSetNotAvailable> {
        Ok(Self)
    }
    type u16x8 = VectorImpl<u16, Wrapping<u16>, U8, Polyfill>;
    type u16x4 = VectorImpl<u16, Wrapping<u16>, U4, Polyfill>;
}

pub trait Vector<B, S, I>:
    Deref<Target = [B]> + DerefMut +
    Sized
{
    // TODO: new_unchecked ‒ aligned, no instruction set checked
    fn new(input: &[B], instruction_set: I) -> Self;
    #[inline]
    fn splat(value: B, instruction_set: I) -> Self
    where
        B: Copy,
        S: ArrayLength<B>,
    {
        let input = iter::repeat(value)
            .take(S::USIZE)
            .collect::<GenericArray<B, S>>();
        Self::new(&input, instruction_set)
    }
}

pub struct VectorImpl<B, R, S, I>
where
    R: inner::Repr<B>,
    S: Unsigned + Mul<R::LANE_MULTIPLYIER> + ArrayLength<R>,
{
    content: GenericArray<R, S>,
    _props: PhantomData<(B, I, <S as Mul<R::LANE_MULTIPLYIER>>::Output)>,
}

impl<B, R, S, I> Vector<B, <S as Mul<R::LANE_MULTIPLYIER>>::Output, I> for VectorImpl<B, R, S, I>
where
    R: inner::Repr<B>,
    S: Unsigned + Mul<R::LANE_MULTIPLYIER> + ArrayLength<R>,
    <S as Mul<R::LANE_MULTIPLYIER>>::Output: ArrayLength<B>,
{
    #[inline]
    fn new(input: &[B], _instruction_set: I) -> Self {
        assert_eq!(
            input.len(),
            S::USIZE * R::LANE_MULTIPLYIER::USIZE,
            "Creating vector from the wrong sized slice",
        );
        unsafe {
            let input: &[R] = slice::from_raw_parts(input.as_ptr().cast(), S::USIZE);
            Self {
                content: GenericArray::clone_from_slice(input),
                _props: PhantomData,
            }
        }
    }

}

impl<B, R, S, I> Deref for VectorImpl<B, R, S, I>
where
    R: inner::Repr<B>,
    S: Unsigned + Mul<R::LANE_MULTIPLYIER> + ArrayLength<R>,
{
    type Target = [B];
    #[inline]
    fn deref(&self) -> &[B] {
        unsafe {
            slice::from_raw_parts(
                self.content.as_ptr().cast(),
                S::USIZE * R::LANE_MULTIPLYIER::USIZE,
            )
        }
    }
}

impl<B, R, S, I> DerefMut for VectorImpl<B, R, S, I>
where
    R: inner::Repr<B>,
    S: Unsigned + Mul<R::LANE_MULTIPLYIER> + ArrayLength<R>,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut [B] {
        unsafe {
            slice::from_raw_parts_mut(
                self.content.as_mut_ptr().cast(),
                S::USIZE * R::LANE_MULTIPLYIER::USIZE,
            )
        }
    }
}

impl<B, R, S, I> Index<usize> for VectorImpl<B, R, S, I>
where
    R: inner::Repr<B>,
    S: Unsigned + Mul<R::LANE_MULTIPLYIER> + ArrayLength<R>,
{
    type Output = B;
    #[inline]
    fn index(&self, idx: usize) -> &B {
        self.deref().index(idx)
    }
}

impl<B, R, S, I> IndexMut<usize> for VectorImpl<B, R, S, I>
where
    R: inner::Repr<B>,
    S: Unsigned + Mul<R::LANE_MULTIPLYIER> + ArrayLength<R>,
{
    #[inline]
    fn index_mut(&mut self, idx: usize) -> &mut B {
        self.deref_mut().index_mut(idx)
    }
}

macro_rules! bin_op_impl {
    ($tr: ident, $meth: ident, $tr_assign: ident, $meth_assign: ident) => {
        impl<B, R, S, I> $tr for VectorImpl<B, R, S, I>
        where
            R: inner::Repr<B> + $tr<Output = R> + Copy,
            S: Unsigned + Mul<R::LANE_MULTIPLYIER> + ArrayLength<R>,
        {
            type Output = Self;
            fn $meth(self, rhs: Self) -> Self {
                let content = self.content.iter()
                    .zip(rhs.content.iter())
                    .map(|(a, b)| $tr::$meth(*a, *b))
                    .collect();
                Self {
                    content,
                    _props: PhantomData,
                }
            }
        }

        impl<B, R, S, I> $tr_assign for VectorImpl<B, R, S, I>
        where
            R: inner::Repr<B> + $tr_assign + Copy,
            S: Unsigned + Mul<R::LANE_MULTIPLYIER> + ArrayLength<R>,
        {
            fn $meth_assign(&mut self, rhs: Self) {
                for (r, s) in self.content.iter_mut().zip(rhs.content.iter()) {
                    $tr_assign::$meth_assign(r, *s);
                }
            }
        }
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
