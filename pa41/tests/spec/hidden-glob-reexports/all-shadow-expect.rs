#![allow(dead_code, unused_imports)]
mod source { pub mod item {} }
pub use source::*;
#[expect(hidden_glob_reexports)]
mod item {}
fn main() {}
