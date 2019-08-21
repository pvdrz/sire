#![feature(rustc_private)]

extern crate rustc;
extern crate rustc_driver;
extern crate rustc_interface;
extern crate syntax;

pub mod analysis;
pub mod eval;
pub mod sir;
