#[cfg(target_arch = "x86")]
pub use core::arch::x86::{self as arch, __m128};
#[cfg(target_arch = "x86_64")]
pub use core::arch::x86_64::{self as arch, __m128};
use std::iter;

use once_cell::sync::Lazy;

#[macro_export]
macro_rules! mv {
    ($(fn $name: ident($($params: tt)*) $(-> $res: ty)? $body: block)*) => {
        $(
            #[multiversion::multiversion(targets = "simd")]
            fn $name($($params)*) $(-> $res)? $body
        )*
    };
}

pub(crate) const SIZE: usize = 10 * 1024 * 1024;
pub(crate) type V = slipstream::f32x4;

pub(crate) fn gen_data() -> (&'static [f32], &'static [f32]) {
    fn inner() -> Vec<f32> {
        iter::repeat_with(rand::random).take(SIZE).collect()
    }
    static CACHED: Lazy<(Vec<f32>, Vec<f32>)> = Lazy::new(|| (inner(), inner()));
    (&CACHED.0, &CACHED.1)
}

pub(crate) fn gen_vecs() -> (&'static [V], &'static [V]) {
    fn inner() -> Vec<V> {
        iter::repeat_with(rand::random)
            .map(|v: [f32; V::LANES]| V::new(&v))
            .take(SIZE / V::LANES)
            .collect()
    }
    static CACHED: Lazy<(Vec<V>, Vec<V>)> = Lazy::new(|| (inner(), inner()));
    (&CACHED.0, &CACHED.1)
}

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub(crate) fn gen_arch_vecs() -> (&'static [__m128], &'static [__m128]) {
    fn inner() -> Vec<__m128> {
        iter::repeat_with(|| {
            let v: [f32; 4] = rand::random();
            unsafe { arch::_mm_loadu_ps(v.as_ptr()) }
        })
        .take(SIZE / 4)
        .collect()
    }

    static CACHED: Lazy<(Vec<__m128>, Vec<__m128>)> = Lazy::new(|| (inner(), inner()));
    (&CACHED.0, &CACHED.1)
}
