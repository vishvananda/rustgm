#![feature(staged_api)]#![stable(feature="alias_fixture",since="1.0.0")]
extern crate self as renamed;
pub mod nested { pub fn child(){} }
