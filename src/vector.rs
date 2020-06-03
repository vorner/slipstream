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
                            result.as_mut_ptr().cast::<B>().add(i),
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
                            result.as_mut_ptr().cast::<B>().add(i),
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

        impl<B, S> Vector for $name<B, S>
        where
            B: inner::Repr + Add<Output = B> + Mul<Output = B> + 'static,
            S: ArrayLength<B> + 'static,
            S::ArrayType: Copy,
        {
            type Base = B;
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
            fn splat(value: B) -> Self {
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
            fn gather_load<I, Idx>(input: I, idx: Idx) -> Self
            where
                I: AsRef<[B]>,
                Idx: AsRef<[usize]>,
            {
                let input = input.as_ref();
                let idx = idx.as_ref();
                assert!(
                    isize::MAX as usize > mem::size_of::<Self>(),
                    "Vector type too huge",
                );
                assert_eq!(Self::LANES, idx.len(), "Gathering vector from wrong number of indexes");
                let mut result = MaybeUninit::<GenericArray<B, S>>::uninit();
                unsafe {
                    for i in 0..Self::LANES {
                        ptr::write(
                            result.as_mut_ptr().cast::<B>().add(i),
                            input[idx[i]],
                        );
                    }
                    Self {
                        content: result.assume_init(),
                    }
                }
            }

            #[inline]
            fn scatter_store<O, Idx>(self, mut output: O, idx: Idx)
            where
                O: AsMut<[B]>,
                Idx: AsRef<[usize]>,
            {
                let output = output.as_mut();
                let idx = idx.as_ref();
                assert_eq!(Self::LANES, idx.len(), "Scattering vector to wrong number of indexes");
                // Check prior to starting the scatter before we write anything. Might be nicer for
                // optimizer + we don't want to do partial scatter.
                let max_len = *idx.iter().max().expect("Must have max on indexes");
                assert!(
                    max_len < output.len(),
                    "Scatter out of bounds ({} >= {})",
                    max_len,
                    output.len(),
                );
                for i in 0..Self::LANES {
                    output[idx[i]] = self[i];
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

        impl<B, S> AsRef<[B]> for $name<B, S>
        where
            B: inner::Repr,
            S: ArrayLength<B>,
            S::ArrayType: Copy,
        {
            #[inline]
            fn as_ref(&self) -> &[B] {
                &self.content
            }
        }

        impl<B, S> AsMut<[B]> for $name<B, S>
        where
            B: inner::Repr,
            S: ArrayLength<B>,
            S::ArrayType: Copy,
        {
            #[inline]
            fn as_mut(&mut self) -> &mut [B] {
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
            B: inner::Repr + MulAssign,
            S: ArrayLength<B>,
            S::ArrayType: Copy,
            Self: Vector<Base = B, Lanes = S>,
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

// TODO: AsRef impls for arrays of fixed size by macros for common sizes

#[cfg(test)]
mod tests {
    use typenum::consts::*;

    use super::*;

    type V = Packed2<u16, U4>;

    #[test]
    #[should_panic(expected = "Creating vector from the wrong sized slice (expected 4, got 3)")]
    fn wrong_size_new() {
        V::new([1, 2, 3]);
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
        let v = V::gather_load(&data, [0, 2, 4, 6]);
        assert_eq!(v.deref(), [1, 3, 5, 7]);
    }

    #[test]
    fn scatter() {
        let v = V::new([1, 2, 3, 4]);
        let mut output = [0; 10];
        v.scatter_store(&mut output, [1, 3, 5, 7]);
        assert_eq!(output, [0, 1, 0, 2, 0, 3, 0, 4, 0, 0]);
    }

    #[test]
    #[should_panic(expected = "index out of bounds")]
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
}
