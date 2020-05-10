#![allow(non_camel_case_types)]

use std::error::Error;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::marker::PhantomData;
use std::ops::{Add, Deref, DerefMut};

mod inner {
    use super::u16x8;

    pub trait InstructionSet: Sized {
        fn add_u16x8(l: u16x8<Self>, r: u16x8<Self>) -> u16x8<Self>;
        unsafe fn new_u16x8(value: [u16; 8]) -> u16x8<Self>;
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct InstructionSetNotAvailable(pub &'static str);

impl Display for InstructionSetNotAvailable {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        write!(fmt, "Instruction set {} not available", self.0)
    }
}

impl Error for InstructionSetNotAvailable {}

pub trait Vector<InstructionSet>: Add + Copy + Debug + DerefMut {
    fn new(value: <Self as Deref>::Target, token: InstructionSet) -> Self;
}

pub trait InstructionSet: Copy + Debug + inner::InstructionSet {
    fn detect() -> Result<Self, InstructionSetNotAvailable>;
}

// It's OK to let users create this one directly, it's safe to use always.
#[derive(Copy, Clone, Debug)]
struct Polyfill;

impl inner::InstructionSet for Polyfill {
    fn add_u16x8(l: u16x8<Self>, r: u16x8<Self>) -> u16x8<Self> {
        unimplemented!()
    }
    unsafe fn new_u16x8(value: [u16; 8]) -> u16x8<Self> {
        unimplemented!()
    }
}

impl InstructionSet for Polyfill {
    fn detect() -> Result<Self, InstructionSetNotAvailable> {
        Ok(Self)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct u16x8<IS>([u16; 8], PhantomData<IS>);

// TODO: Default to static detect
impl<IS> Deref for u16x8<IS> {
    type Target = [u16; 8];
    fn deref(&self) -> &[u16; 8] {
        &self.0
    }
}

impl<IS> DerefMut for u16x8<IS> {
    fn deref_mut(&mut self) -> &mut [u16; 8] {
        &mut self.0
    }
}

impl<IS: inner::InstructionSet> Add for u16x8<IS> {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        IS::add_u16x8(self, other)
    }
}

impl<IS: InstructionSet> Vector<IS> for u16x8<IS> {
    fn new(value: [u16; 8], _token: IS) -> Self {
        unsafe {
            IS::new_u16x8(value)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn add<S: InstructionSet>() {
        let token = S::detect().unwrap();
        let x = u16x8::new([0; 8], token);
        let y = u16x8::new([1; 8], token);
        let z = x + y;
    }
}
