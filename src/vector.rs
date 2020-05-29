use core::marker::PhantomData;
use core::mem::MaybeUninit;
use core::ptr;
use core::slice;
use core::ops::*;

use generic_array::{ArrayLength, GenericArray};
use typenum::marker_traits::Unsigned;

use crate::{inner, Vector};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct VectorImpl<B, R, S, I>
where
    R: inner::Repr<B>,
    S: Unsigned + Mul<R::LaneMultiplyier> + ArrayLength<R>,
    S::ArrayType: Copy,
{
    content: GenericArray<R, S>,
    _props: PhantomData<(B, I, <S as Mul<R::LaneMultiplyier>>::Output)>,
}

impl<B, R, S, I> Vector<B, I> for VectorImpl<B, R, S, I>
where
    R: inner::Repr<B>,
    S: Unsigned + Mul<R::LaneMultiplyier> + ArrayLength<R>,
    S::ArrayType: Copy,
    <S as Mul<R::LaneMultiplyier>>::Output: ArrayLength<B>,
{
    type Lanes = <S as Mul<R::LaneMultiplyier>>::Output;
    #[inline]
    fn new(input: &[B], _instruction_set: I) -> Self {
        assert_eq!(
            input.len(),
            S::USIZE * R::LaneMultiplyier::USIZE,
            "Creating vector from the wrong sized slice",
        );
        unsafe {
            let content = ptr::read_unaligned(input.as_ptr().cast());
            Self {
                content,
                _props: PhantomData,
            }
        }
    }
}

impl<B, R, S, I> Deref for VectorImpl<B, R, S, I>
where
    R: inner::Repr<B>,
    S: Unsigned + Mul<R::LaneMultiplyier> + ArrayLength<R>,
    S::ArrayType: Copy,
{
    type Target = [B];
    #[inline]
    fn deref(&self) -> &[B] {
        unsafe {
            slice::from_raw_parts(
                self.content.as_ptr().cast(),
                S::USIZE * R::LaneMultiplyier::USIZE,
            )
        }
    }
}

impl<B, R, S, I> DerefMut for VectorImpl<B, R, S, I>
where
    R: inner::Repr<B>,
    S: Unsigned + Mul<R::LaneMultiplyier> + ArrayLength<R>,
    S::ArrayType: Copy,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut [B] {
        unsafe {
            slice::from_raw_parts_mut(
                self.content.as_mut_ptr().cast(),
                S::USIZE * R::LaneMultiplyier::USIZE,
            )
        }
    }
}

impl<B, R, S, I> Index<usize> for VectorImpl<B, R, S, I>
where
    R: inner::Repr<B>,
    S: Unsigned + Mul<R::LaneMultiplyier> + ArrayLength<R>,
    S::ArrayType: Copy,
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
    S: Unsigned + Mul<R::LaneMultiplyier> + ArrayLength<R>,
    S::ArrayType: Copy,
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
            S: Unsigned + Mul<R::LaneMultiplyier> + ArrayLength<R>,
            S::ArrayType: Copy,
        {
            type Output = Self;
            #[inline]
            fn $meth(self, rhs: Self) -> Self {
                unsafe {
                    let mut result = MaybeUninit::<GenericArray<R, S>>::uninit();
                    for i in 0..S::USIZE {
                        ptr::write(
                            result.as_mut_ptr().cast::<R>().offset(i as isize),
                            $tr::$meth(self.content[i], rhs.content[i]),
                            );
                    }
                    Self {
                        content: result.assume_init(),
                        _props: PhantomData,
                    }
                }
            }
        }

        impl<B, R, S, I> $tr_assign for VectorImpl<B, R, S, I>
        where
            R: inner::Repr<B> + $tr_assign + Copy,
            S: Unsigned + Mul<R::LaneMultiplyier> + ArrayLength<R>,
            S::ArrayType: Copy,
        {
            #[inline]
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

macro_rules! una_op_impl {
    ($tr: ident, $meth: ident) => {
        impl<B, R, S, I> $tr for VectorImpl<B, R, S, I>
        where
            R: inner::Repr<B> + $tr<Output = R> + Copy,
            S: Unsigned + Mul<R::LaneMultiplyier> + ArrayLength<R>,
            S::ArrayType: Copy,
        {
            type Output = Self;
            fn $meth(self) -> Self {
                let content = self.content
                    .iter()
                    .copied()
                    .map($tr::$meth)
                    .collect();
                Self {
                    content,
                    _props: PhantomData,
                }
            }
        }
    }
}

una_op_impl!(Neg, neg);
una_op_impl!(Not, not);
