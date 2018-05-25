// TODO: no_std
// TODO: stdsimd crate
#![feature(stdsimd)]

extern crate faster;
#[cfg(test)]
#[macro_use]
extern crate proptest;

pub mod ascii;
pub mod ext;
pub mod utf8;
