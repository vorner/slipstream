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

## Example

As a very simple example, imagine that the crux of the application's performance
is summing a huge array of floats and we have this code:

```rust
fn compute(d: &[f32]) -> f32 {
    d.iter().sum()
}
```

Now, one could rewrite it to something like this, using manual vectorization:

```rust
use core::arch::x86_64 as arch;

unsafe fn compute_sse(d: &[f32]) -> f32 {
    let mut result = arch::_mm_setzero_ps();
    let iter = data.chunks_exact(4);
    let remainder = iter.remainder().iter().sum::<f32>();
    for v in iter {
        result = arch::_mm_add_ps(result, arch::_mm_loadu_ps(v.as_ptr()));
    }

    let result: [f32; 4] = mem::transmute(result);
    let result = result.iter().sum::<f32>() + remainder;
}
```

And while this does result in significant speedup, it's also much less readable,
one has to allow using unsafe through the application logic and is not portable
(it won't run on anything that's not Intel and it won't take advantage of newer
and better vector instructions even there). These downside usually make it not
worth pursuing for more complex algorithms.

Using `slipstream`, one can also write this:

```rust
fn compute_slipstream(d: &[f32]) -> f32 {
    // Will split the data into vectors of 4 lanes, padding the last one with
    // the lanes from the provided parameter.
    d.vectorize_pad(f32x4::default())
        // Sum the vectors into a final vector
        .sum::<f32x4>()
        // Sum the lanes of the vectors together.
        .horizontal_sum()
}
```

This is still longer and more complex than the original, but seems much more
manageable than the manual version. It's also portable and might provide some
speedup on platforms that don't have any vector instructions. Using the right
annotations on the function, one is also able to generate multiple versions and
dispatch the one that takes advantage of the newest and shiniest instructions
the CPU supports at runtime.

Corresponding benchmarks on i5-8265U suggest that this version comes close to
the manual one. Indeed, there are similar variants that are even faster.

```
test sum::basic                               ... bench:  11,707,693 ns/iter (+/- 261,428)
test sum::manual_sse_convert                  ... bench:   3,000,906 ns/iter (+/- 535,041)
test sum::vectorize_pad_default               ... bench:   3,141,834 ns/iter (+/- 81,376)
```

Note: to re-run the benchmarks as above, use `type V = f32x4` in
`benches/utils.rs`.

Warning: Floats are not associative. The first, manual, version may produce
slightly different results because of rounding errors.

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

If you want to work on anything bigger, it's a good idea to open an issue on the
repository to both discuss it first and to reserve the task.

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
