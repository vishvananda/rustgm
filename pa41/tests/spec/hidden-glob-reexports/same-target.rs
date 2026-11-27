#![allow(dead_code, unused_imports)]
mod source { pub mod item {} pub mod visible {} }
pub use source::*;
#[expect(hidden_glob_reexports)]
use source::item;
fn main() {}
