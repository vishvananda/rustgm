#![allow(dead_code, unused_imports)]
#![feature(decl_macro)]
mod source { pub macro item() {} }
pub use source::*;
#[expect(hidden_glob_reexports)]
mod item {}
fn main() {}
