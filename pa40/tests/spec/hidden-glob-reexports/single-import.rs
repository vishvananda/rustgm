#![allow(dead_code, unused_imports)]
mod source { pub mod item {} pub mod visible {} }
mod private { pub mod item {} }
pub use source::*;
#[expect(hidden_glob_reexports)]
use private::item;
fn main() {}
