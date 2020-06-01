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
