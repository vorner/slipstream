use core::iter::{Product, Sum};
use core::mem::{self, MaybeUninit};
use core::ptr;
use core::ops::*;

use generic_array::{ArrayLength, GenericArray};
use typenum::marker_traits::Unsigned;

use crate::{inner, Vector};

macro_rules! bin_op_impl {
    ($name: ident, $tr: ident, $meth: ident, $tr_assign: ident, $meth_assign: ident) => {
        impl<B, S> $tr for $name<B, S>
        where
            B: inner::Repr + $tr<Output = B> + Copy,
            S: ArrayLength<B>,
            S::ArrayType: Copy,
        {
            type Output = Self;
            #[inline]
            fn $meth(self, rhs: Self) -> Self {
                unsafe {
                    let mut result = MaybeUninit::<GenericArray<B, S>>::uninit();
                    for i in 0..S::USIZE {
                        ptr::write(
                            result.as_mut_ptr().cast::<B>().offset(i as isize),
                            $tr::$meth(self.content[i], rhs.content[i]),
                        );
                    }
                    Self {
                        content: result.assume_init(),
                    }
                }
            }
        }

        impl<B, S> $tr_assign for $name<B, S>
        where
            B: inner::Repr + $tr_assign + Copy,
            S: ArrayLength<B>,
            S::ArrayType: Copy,
        {
            #[inline]
            fn $meth_assign(&mut self, rhs: Self) {
                for i in 0..S::USIZE {
                    $tr_assign::$meth_assign(&mut self.content[i], rhs.content[i]);
                }
            }
        }
    }
}

macro_rules! una_op_impl {
    ($name: ident, $tr: ident, $meth: ident) => {
        impl<B, S> $tr for $name<B, S>
        where
            B: inner::Repr + $tr<Output = B> + Copy,
            S: Unsigned + ArrayLength<B>,
            S::ArrayType: Copy,
        {
            type Output = Self;
            #[inline]
            fn $meth(self) -> Self {
                unsafe {
                    let mut result = MaybeUninit::<GenericArray<B, S>>::uninit();
                    for i in 0..S::USIZE {
                        ptr::write(
                            result.as_mut_ptr().cast::<B>().offset(i as isize),
                            $tr::$meth(self.content[i]),
                        );
                    }
                    Self {
                        content: result.assume_init(),
                    }
                }
            }
        }
    }
}

macro_rules! vector_impl {
    ($name: ident, $align: expr) => {
        #[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
        #[repr(C, align($align))]
        pub struct $name<B, S>
        where
            B: inner::Repr,
            S: ArrayLength<B>,
            S::ArrayType: Copy,
        {
            content: GenericArray<B, S>,
        }

        impl<B, S> Vector<B> for $name<B, S>
        where
            B: inner::Repr + Add<Output = B> + Mul<Output = B> + 'static,
            S: ArrayLength<B> + 'static,
            S::ArrayType: Copy,
        {
            type Lanes = S;
            #[inline]
            unsafe fn new_unchecked(input: *const B) -> Self {
                assert!(
                    isize::MAX as usize > mem::size_of::<Self>(),
                    "Vector type too huge",
                );

                Self {
                    content: ptr::read(input.cast()),
                }
            }

            #[inline]
            fn splat(value: B) -> Self
            where
                B: Clone
            {
                assert!(
                    isize::MAX as usize > mem::size_of::<Self>(),
                    "Vector type too huge",
                );
                let mut result = MaybeUninit::<GenericArray<B, S>>::uninit();
                unsafe {
                    for i in 0..S::USIZE {
                        ptr::write(result.as_mut_ptr().cast::<B>().offset(i as isize), value);
                    }
                    Self {
                        content: result.assume_init(),
                    }
                }
            }

            #[inline]
            fn horizontal_sum(self) -> B {
                #[inline(always)]
                fn inner<B: Copy + Add<Output = B>>(d: &[B]) -> B {
                    if d.len() == 1 {
                        d[0]
                    } else {
                        let mid = d.len() / 2;
                        inner(&d[..mid]) + inner(&d[mid..])
                    }
                }
                inner(&self.content)
            }

            #[inline]
            fn horizontal_product(self) -> B {
                #[inline(always)]
                fn inner<B: Copy + Mul<Output = B>>(d: &[B]) -> B {
                    if d.len() == 1 {
                        d[0]
                    } else {
                        let mid = d.len() / 2;
                        inner(&d[..mid]) * inner(&d[mid..])
                    }
                }
                inner(&self.content)
            }
        }

        impl<B, S> Deref for $name<B, S>
        where
            B: inner::Repr,
            S: ArrayLength<B>,
            S::ArrayType: Copy,
        {
            type Target = [B];
            #[inline]
            fn deref(&self) -> &[B] {
                &self.content
            }
        }

        impl<B, S> DerefMut for $name<B, S>
        where
            B: inner::Repr,
            S: ArrayLength<B>,
            S::ArrayType: Copy,
        {
            #[inline]
            fn deref_mut(&mut self) -> &mut [B] {
                &mut self.content
            }
        }

        impl<B, S> Index<usize> for $name<B, S>
        where
            B: inner::Repr,
            S: ArrayLength<B>,
            S::ArrayType: Copy,
        {
            type Output = B;
            #[inline]
            fn index(&self, idx: usize) -> &B {
                &self.content[idx]
            }
        }

        impl<B, S> IndexMut<usize> for $name<B, S>
        where
            B: inner::Repr,
            S: ArrayLength<B>,
            S::ArrayType: Copy,
        {
            #[inline]
            fn index_mut(&mut self, idx: usize) -> &mut B {
                &mut self.content[idx]
            }
        }

        impl<B, S> Sum for $name<B, S>
        where
            B: inner::Repr + AddAssign,
            S: ArrayLength<B>,
            S::ArrayType: Copy,
            Self: Default,
        {
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

        impl<B, S> Product for $name<B, S>
        where
            B: Copy + inner::Repr + MulAssign,
            S: ArrayLength<B>,
            S::ArrayType: Copy,
            Self: Vector<B>,
        {
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

        bin_op_impl!($name, Add, add, AddAssign, add_assign);
        bin_op_impl!($name, Sub, sub, SubAssign, sub_assign);
        bin_op_impl!($name, Mul, mul, MulAssign, mul_assign);
        bin_op_impl!($name, Div, div, DivAssign, div_assign);
        bin_op_impl!($name, Rem, rem, RemAssign, rem_assign);
        bin_op_impl!($name, BitAnd, bitand, BitAndAssign, bitand_assign);
        bin_op_impl!($name, BitOr, bitor, BitOrAssign, bitor_assign);
        bin_op_impl!($name, BitXor, bitxor, BitXorAssign, bitxor_assign);
        bin_op_impl!($name, Shl, shl, ShlAssign, shl_assign);
        bin_op_impl!($name, Shr, shr, ShrAssign, shr_assign);

        una_op_impl!($name, Neg, neg);
        una_op_impl!($name, Not, not);
    }
}

vector_impl!(Packed1, 1);
vector_impl!(Packed2, 2);
vector_impl!(Packed4, 4);
vector_impl!(Packed8, 8);
vector_impl!(Packed16, 16);
vector_impl!(Packed32, 32);

#[cfg(test)]
mod tests {
    use typenum::consts::*;

    use super::*;

    #[test]
    #[should_panic(expected = "Creating vector from the wrong sized slice (expected 4, got 3)")]
    fn wrong_size_new() {
        type V = Packed2<u16, U4>;

        V::new(&[1, 2, 3]);
    }
}
