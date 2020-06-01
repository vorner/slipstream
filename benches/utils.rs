use std::iter;

use impatient::prelude::*;

#[macro_export]
macro_rules! mv {
    ($(fn $name: ident($($params: tt)*) $(-> $res: ty)? $body: block)*) => {
        $(
            #[multiversion]
            #[clone(target = "[x86|x86_64]+sse+sse2+sse3+sse4.1+avx+avx2")]
            #[clone(target = "[x86|x86_64]+sse+sse2+sse3+sse4.1+avx")]
            #[clone(target = "[x86|x86_64]+sse+sse2+sse3+sse4.1")]
            #[clone(target = "[arm|aarch64]+neon")]
            fn $name($($params)*) $(-> $res)? $body
        )*
    };
}

pub(crate) const SIZE: usize = 10*1024*1024;
pub(crate) type V = impatient::f32x16;

pub(crate) fn gen_data() -> Vec<f32> {
    iter::repeat_with(rand::random)
        .take(SIZE)
        .collect()
}

pub(crate) fn gen_vecs() -> Vec<V> {
    iter::repeat_with(rand::random)
        .map(|v: [f32; V::LANES]| V::new(&v))
        .take(SIZE / V::LANES)
        .collect()
}

#[cfg(target_arch = "x86_64")]
pub(crate) fn gen_arch_vecs() -> Vec<core::arch::x86_64::__m128> {
    use core::arch::x86_64 as arch;

    iter::repeat_with(|| {
            let v: [f32; 4] = rand::random();
            unsafe { arch::_mm_loadu_ps(v.as_ptr()) }
        })
        .take(SIZE / 4)
        .collect()
}
