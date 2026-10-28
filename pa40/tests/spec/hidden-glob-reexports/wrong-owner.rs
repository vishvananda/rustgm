#![allow(dead_code, unused_imports)]
mod source { pub mod item {} pub mod visible {} }
mod private { #[expect(hidden_glob_reexports)] pub mod item {} }
pub use source::*;
use private::item;
fn main() {}
