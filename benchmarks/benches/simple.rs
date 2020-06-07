#![feature(test)]
// These two are needed when benchmarking for arm
#![feature(aarch64_target_feature)]
#![feature(arm_target_feature)]
#![feature(stdsimd)]
// The lint comes from somewhere inside macros, no idea why :-(
#![allow(unused_braces)]

extern crate test;

mod utils;

mod dot_product;
mod life;
mod sum;
