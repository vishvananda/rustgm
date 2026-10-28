#![allow(dead_code, unused_imports)]
mod source { pub mod item {} pub mod other {} }
mod hidden { pub use super::source::*; #[expect(hidden_glob_reexports)] mod item {} }
fn main() {}
