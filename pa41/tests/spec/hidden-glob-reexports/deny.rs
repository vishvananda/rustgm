#![allow(dead_code, unused_imports)]
#![deny(hidden_glob_reexports)]
mod source { pub mod item {} pub mod visible {} }
pub use source::*;
mod item {}
fn main() {}
