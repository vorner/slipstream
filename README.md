# Slipstream

[![Travis Build Status](https://api.travis-ci.org/vorner/slipstream.png?branch=master)](https://travis-ci.org/vorner/slipstream)

This library helps writing code in a way that incentives the compiler to
optimize the results better (without really doing anything itself).

Modern compilers, including `rustc`, are able to come up with impressive ways to
speed up the resulting code, using techniques like loop unrolling and
autovectorization, routinely outperforming what one would hand-craft.
Nevertheless, each optimisation has some assumptions that must be proven to hold
before it can be applied.

This library offers „vector“ types, like `u16x8`, which act in a very similar
way as little fixed-sized arrays (in this case it would be `[u16; 8]`), but with
arithmetics defined for them. They also enforce alignment of the whole vectors.
Therefore, one can write the algorithm in a way that works on these groups of
data and make it easier for the compiler to prove the assumptions. This can
result in multiple factor speed ups by giving the compiler these proofs „for
free“ and allowing it to apply aggressive optimizations.

The API is inspired by the [`packed_simd`] and [`faster`] crates, but as it
relies on the autovectorizer instead of using explicit SIMD instructions, it
works on stable rust, allows speed ups even on platforms that don't have
explicit SIMD support from the rust standard library (or no SIMD support at
all).

The downside is the optimizations are not *guaranteed*. While it oftentimes
produces results competitive or even better than hand-crafted vectorized code,
a small change to surrounding code can also lead to much worse results.  You're
advised to apply this to only tight loops with enough data to crunch and to
measure the performance.

It goes well together with function multiversioning, see for example the
[`multiversion`] crate.

More details can be found in the [documentation], including tips for effective
use and what to try if the performance isn't as good as expected.

## Help wanted

It is an open source library and help in developing it is welcome. There are
some areal where Your contribution would be especially appreciated:

* Feedback about the API, documentation and generally how well it is usable.
* Implementing missing APIs: While a lot is covered already, there are areas
  that are still missing. I know of:
  - Some way to convert between different sizes of the base type (eg. `f32x4 ->
    f64x4`).
  - Various methods on types that are present on the base types ‒ trigonometric
    functions on floats, rounding, absolute values, number of set/unset bits on
    unsigned integers...
  - Vector-scalar multiplications. It is currently possible to do eg
    `f32x2::splat(-1.0) * f32x2::new([1, 2])`, but it would be more comfortable
    if it could be just written as `-1.0 * f32x2::new([1, 2])`.
* Use cases and benchmarks: if you can come up with a simple, well-vectorizable
  problem and submit it as a benchmark, it helps keeping and improving the
  performance of the library. Both cases where the library performs well and
  where it *doesn't* are good to have (the latter could be considered bugs of a
  kind). Optimally, if such benchmark contains a naïve implementation (without
  this library), implementation using this library (possibly in multiple
  variations) and a hand-written vectorized code with the platform specific
  intrinsics. But if any of these are missing (for example because it would be
  too much work to write the manually vectorized code), it's still better than
  nothing.
* Improving performance: While it is the compiler that makes the program go
  fast, how good the compiler is in the job highly depends on if it can „see
  through“ the code. If you can tweak implementation of some method in a way
  that's more understandable and transparent to the compiler, it is great. Most
  of the code was written as fast as possible and only some tweaking was done
  for now. For example, the `vectorize_pad` method seems surprisingly slow,
  ideally it would produce code with comparable speed to `vectorize`.
* Dealing with unsafe: At many places, the library uses `unsafe` code. This was
  oftentimes written that way because of performance ‒ for example, initializing
  the `GenericArray` from an iterator prevented a lot of optimisations and led
  to significantly inferior performance. Optimally, each such `unsafe` code
  would get replaced by safe code, or would get a comment explaining/proving
  that it is indeed safe.

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms
or conditions.

[`packed_simd`]: https://crates.io/crates/packed_simd
[`faster`]: https://crates.io/crates/faster
[`multiversion`]: https://crates.io/crates/multiversion
[documentation]: https://docs.rs/slipstream
