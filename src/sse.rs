use core::num::Wrapping;
#[cfg(target_arch = "x86")]
use core::arch::x86 as arch;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64 as arch;
use core::ops::*;

use arch::__m128i;

use typenum::consts::*;

use crate::{InstructionSet, InstructionSetNotAvailable, VectorImpl};
use crate::inner::Repr;

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct u32v(__m128i);

unsafe impl Repr<u32> for u32v {
    type LANE_MULTIPLYIER = U4;
}

impl Add for u32v {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        #[inline]
        #[target_feature(enable = "sse2", enable = "sse4.1")]
        unsafe fn inner(lhs: u32v, rhs: u32v) -> u32v {
            u32v(arch::_mm_add_epi32(lhs.0, rhs.0))
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
        #[target_feature(enable = "sse2", enable = "sse4.1")]
        unsafe fn inner(lhs: u32v, rhs: u32v) -> u32v {
            u32v(arch::_mm_sub_epi32(lhs.0, rhs.0))
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
        #[target_feature(enable = "sse2", enable = "sse4.1")]
        unsafe fn inner(lhs: u32v, rhs: u32v) -> u32v {
            u32v(arch::_mm_mullo_epi32(lhs.0, rhs.0))
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
pub struct u16v(__m128i);

unsafe impl Repr<u16> for u16v {
    type LANE_MULTIPLYIER = U8;
}

impl Add for u16v {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        #[inline]
        #[target_feature(enable = "sse2", enable = "sse4.1")]
        unsafe fn inner(lhs: u16v, rhs: u16v) -> u16v {
            u16v(arch::_mm_add_epi16(lhs.0, rhs.0))
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
        #[target_feature(enable = "sse2", enable = "sse4.1")]
        unsafe fn inner(lhs: u16v, rhs: u16v) -> u16v {
            u16v(arch::_mm_sub_epi16(lhs.0, rhs.0))
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
        #[target_feature(enable = "sse2", enable = "sse4.1")]
        unsafe fn inner(lhs: u16v, rhs: u16v) -> u16v {
            u16v(arch::_mm_mullo_epi16(lhs.0, rhs.0))
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
pub struct Sse4_1(());

impl crate::inner::InstructionSet for Sse4_1 { }

impl InstructionSet for Sse4_1 {
    fn detect() -> Result<Self, InstructionSetNotAvailable> {
        if is_x86_feature_detected!("sse2") {
            Ok(Self(()))
        } else {
            Err(InstructionSetNotAvailable("SSE2"))
        }
    }

    type u16x4 = VectorImpl<u16, Wrapping<u16>, U4, Sse4_1>;
    type u16x8 = VectorImpl<u16, u16v, U1, Sse4_1>;
    type u32x16 = VectorImpl<u32, u32v, U4, Sse4_1>;
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use crate::Polyfill;
    use super::*;

    proptest! {
        #[test]
        fn add_u32(a: [u32; 16], b: [u32; 16]) {
            #[inline]
            #[target_feature(enable = "sse2", enable = "sse4.1")]
            unsafe fn inner(sse: Sse4_1, poly: Polyfill, a: [u32; 16], b: [u32; 16]) {
                let sa = sse.load_u32x16(&a);
                let sb = sse.load_u32x16(&b);
                let sc = sa + sb;
                let pa = poly.load_u32x16(&a);
                let pb = poly.load_u32x16(&b);
                let pc = pa + pb;
                assert_eq!(sc.deref(), pc.deref());
            }
            let sse = Sse4_1::detect().unwrap();
            let poly = Polyfill::detect().unwrap();
            unsafe {
                inner(sse, poly, a, b);
            }
        }

        #[test]
        fn sub_u32(a: [u32; 16], b: [u32; 16]) {
            #[inline]
            #[target_feature(enable = "sse2", enable = "sse4.1")]
            unsafe fn inner(sse: Sse4_1, poly: Polyfill, a: [u32; 16], b: [u32; 16]) {
                let sa = sse.load_u32x16(&a);
                let sb = sse.load_u32x16(&b);
                let sc = sa - sb;
                let pa = poly.load_u32x16(&a);
                let pb = poly.load_u32x16(&b);
                let pc = pa - pb;
                assert_eq!(sc.deref(), pc.deref());
            }
            let sse = Sse4_1::detect().unwrap();
            let poly = Polyfill::detect().unwrap();
            unsafe {
                inner(sse, poly, a, b);
            }
        }

        #[test]
        fn mul_u32(a: [u32; 16], b: [u32; 16]) {
            #[inline]
            #[target_feature(enable = "sse2", enable = "sse4.1")]
            unsafe fn inner(sse: Sse4_1, poly: Polyfill, a: [u32; 16], b: [u32; 16]) {
                let sa = sse.load_u32x16(&a);
                let sb = sse.load_u32x16(&b);
                let sc = sa * sb;
                let pa = poly.load_u32x16(&a);
                let pb = poly.load_u32x16(&b);
                let pc = pa * pb;
                assert_eq!(sc.deref(), pc.deref());
            }

            let sse = Sse4_1::detect().unwrap();
            let poly = Polyfill::detect().unwrap();
            unsafe {
                inner(sse, poly, a, b);
            }
        }

        #[test]
        fn add_u16(a: [u16; 8], b: [u16; 8]) {
            #[inline]
            #[target_feature(enable = "sse2", enable = "sse4.1")]
            unsafe fn inner(sse: Sse4_1, poly: Polyfill, a: [u16; 8], b: [u16; 8]) {
                let sa = sse.load_u16x8(&a);
                let sb = sse.load_u16x8(&b);
                let sc = sa + sb;
                let pa = poly.load_u16x8(&a);
                let pb = poly.load_u16x8(&b);
                let pc = pa + pb;
                assert_eq!(sc.deref(), pc.deref());
            }
            let sse = Sse4_1::detect().unwrap();
            let poly = Polyfill::detect().unwrap();
            unsafe {
                inner(sse, poly, a, b);
            }
        }

        #[test]
        fn sub_u16(a: [u16; 8], b: [u16; 8]) {
            #[inline]
            #[target_feature(enable = "sse2", enable = "sse4.1")]
            unsafe fn inner(sse: Sse4_1, poly: Polyfill, a: [u16; 8], b: [u16; 8]) {
                let sa = sse.load_u16x8(&a);
                let sb = sse.load_u16x8(&b);
                let sc = sa - sb;
                let pa = poly.load_u16x8(&a);
                let pb = poly.load_u16x8(&b);
                let pc = pa - pb;
                assert_eq!(sc.deref(), pc.deref());
            }
            let sse = Sse4_1::detect().unwrap();
            let poly = Polyfill::detect().unwrap();
            unsafe {
                inner(sse, poly, a, b);
            }
        }

        #[test]
        fn mul_u16(a: [u16; 8], b: [u16; 8]) {
            #[inline]
            #[target_feature(enable = "sse2", enable = "sse4.1")]
            unsafe fn inner(sse: Sse4_1, poly: Polyfill, a: [u16; 8], b: [u16; 8]) {
                let sa = sse.load_u16x8(&a);
                let sb = sse.load_u16x8(&b);
                let sc = sa * sb;
                let pa = poly.load_u16x8(&a);
                let pb = poly.load_u16x8(&b);
                let pc = pa * pb;
                assert_eq!(sc.deref(), pc.deref());
            }

            let sse = Sse4_1::detect().unwrap();
            let poly = Polyfill::detect().unwrap();
            unsafe {
                inner(sse, poly, a, b);
            }
        }
    }
}
