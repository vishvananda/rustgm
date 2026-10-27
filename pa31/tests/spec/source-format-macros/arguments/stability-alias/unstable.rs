#![feature(staged_api)]#![unstable(feature="alias_fixture",issue="none")]
extern crate self as renamed;
pub mod nested { pub fn child(){} }
