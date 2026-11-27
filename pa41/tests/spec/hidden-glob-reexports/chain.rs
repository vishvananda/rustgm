#![allow(dead_code, unused_imports)]
mod source { pub mod item {} pub mod other {} }
mod first { pub use super::source::*; #[expect(hidden_glob_reexports)] mod item {} }
mod second { pub use super::first::other; }
pub use second::other;
fn main() {}
