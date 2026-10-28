#![allow(dead_code, unused_imports)]
#![forbid(hidden_glob_reexports)]
mod source { pub mod item {} }
pub use source::*;
#[expect(hidden_glob_reexports)]
mod item {}
fn main() {}
