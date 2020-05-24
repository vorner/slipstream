use core::num::Wrapping;
#[cfg(target_arch = "x86")]
use core::arch::x86 as arch;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64 as arch;
use core::ops::*;

use arch::__m256i;

use typenum::consts::*;

use crate::{InstructionSet, InstructionSetNotAvailable};
use crate::inner::Repr;
use crate::vector::VectorImpl;

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct u32v(__m256i);

unsafe impl Repr<u32> for u32v {
    type LaneMultiplyier = U8;
}

impl Add for u32v {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        #[inline]
        #[target_feature(enable = "avx2")]
        unsafe fn inner(lhs: u32v, rhs: u32v) -> u32v {
            u32v(arch::_mm256_add_epi32(lhs.0, rhs.0))
        }
        unsafe {
            inner(self, rhs)
        }
    }
}

impl AddAssign for u32v {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sub for u32v {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        #[inline]
        #[target_feature(enable = "avx2")]
        unsafe fn inner(lhs: u32v, rhs: u32v) -> u32v {
            u32v(arch::_mm256_sub_epi32(lhs.0, rhs.0))
        }
        unsafe {
            inner(self, rhs)
        }
    }
}

impl SubAssign for u32v {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl Mul for u32v {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Self) -> Self {
        #[inline]
        #[target_feature(enable = "avx2")]
        unsafe fn inner(lhs: u32v, rhs: u32v) -> u32v {
            u32v(arch::_mm256_mullo_epi32(lhs.0, rhs.0))
        }
        unsafe {
            inner(self, rhs)
        }
    }
}

impl MulAssign for u32v {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct u16v(__m256i);

unsafe impl Repr<u16> for u16v {
    type LaneMultiplyier = U16;
}

impl Add for u16v {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        #[inline]
        #[target_feature(enable = "avx2")]
        unsafe fn inner(lhs: u16v, rhs: u16v) -> u16v {
            u16v(arch::_mm256_add_epi16(lhs.0, rhs.0))
        }
        unsafe {
            inner(self, rhs)
        }
    }
}

impl AddAssign for u16v {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sub for u16v {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        #[inline]
        #[target_feature(enable = "avx2")]
        unsafe fn inner(lhs: u16v, rhs: u16v) -> u16v {
            u16v(arch::_mm256_sub_epi16(lhs.0, rhs.0))
        }
        unsafe {
            inner(self, rhs)
        }
    }
}

impl SubAssign for u16v {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl Mul for u16v {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Self) -> Self {
        #[inline]
        #[target_feature(enable = "avx2")]
        unsafe fn inner(lhs: u16v, rhs: u16v) -> u16v {
            u16v(arch::_mm256_mullo_epi16(lhs.0, rhs.0))
        }
        unsafe {
            inner(self, rhs)
        }
    }
}

impl MulAssign for u16v {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Avx2(());

impl crate::inner::InstructionSet for Avx2 { }

impl InstructionSet for Avx2 {
    fn detect() -> Result<Self, InstructionSetNotAvailable> {
        if is_x86_feature_detected!("avx2") {
            Ok(Self(()))
        } else {
            Err(InstructionSetNotAvailable("AVX2"))
        }
    }

    type u16x8 = u16x8;
    type u16x4 = u16x4;
    type u32x16 = u32x16;

    type u16s = u16s;
    type u32s = u32s;
}

pub type u16x4 = VectorImpl<u16, Wrapping<u16>, U4, Avx2>;
pub type u16x8 = VectorImpl<u16, crate::sse::u16v, U1, Avx2>;
pub type u32x16 = VectorImpl<u32, u32v, U2, Avx2>;

pub type u16s = VectorImpl<u16, u16v, U1, Avx2>;
pub type u32s = VectorImpl<u32, u32v, U1, Avx2>;

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use crate::polyfill::{self as p, Polyfill};
    use super::*;

    macro_rules! tst {
        ($meth: ident, $t_u16: ident, $t_u32: ident) => {
            proptest! {
                #[test]
                fn $t_u32(a: [u32; 16], b: [u32; 16]) {
                    #[inline]
                    #[target_feature(enable = "avx", enable = "avx2")]
                    unsafe fn inner(avx: Avx2, a: [u32; 16], b: [u32; 16]) {
                        let poly = Polyfill::detect().unwrap();
                        let sa: u32x16 = avx.load(&a);
                        let sb: u32x16 = avx.load(&b);
                        let sc = sa.$meth(sb);
                        let pa: p::u32x16 = poly.load(&a);
                        let pb: p::u32x16 = poly.load(&b);
                        let pc = pa.$meth(pb);
                        assert_eq!(sc.deref(), pc.deref());
                    }

                    if let Ok(avx) = Avx2::detect() {
                        unsafe {
                            inner(avx, a, b);
                        }
                    }
                }

                #[test]
                fn $t_u16(a: [u16; 8], b: [u16; 8]) {
                    #[inline]
                    #[target_feature(enable = "avx", enable = "avx2")]
                    unsafe fn inner(avx: Avx2, a: [u16; 8], b: [u16; 8]) {
                        let poly = Polyfill::detect().unwrap();
                        let sa: u16x8 = avx.load(&a);
                        let sb: u16x8 = avx.load(&b);
                        let sc = sa.$meth(sb);
                        let pa: p::u16x8 = poly.load(&a);
                        let pb: p::u16x8 = poly.load(&b);
                        let pc = pa.$meth(pb);
                        assert_eq!(sc.deref(), pc.deref());
                    }

                    if let Ok(avx) = Avx2::detect() {
                        unsafe {
                            inner(avx, a, b);
                        }
                    }
                }
            }
        }
    }

    tst!(mul, mul_u16, mul_u32);
    tst!(add, add_u16, add_u32);
    tst!(sub, sub_u16, sub_u32);
}
