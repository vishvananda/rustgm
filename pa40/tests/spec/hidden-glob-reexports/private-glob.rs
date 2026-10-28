#![allow(dead_code, unused_imports)]
mod source { pub mod item {} }
use source::*;
#[expect(hidden_glob_reexports)]
mod item {}
fn main() {}
