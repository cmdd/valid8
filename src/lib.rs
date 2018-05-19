#![feature(stdsimd)]

extern crate faster;
#[cfg(test)]
#[macro_use]
extern crate proptest;

pub mod ascii;
pub mod utf8;
