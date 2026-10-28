#![allow(dead_code, unused_imports)]
mod source { pub struct Item; pub mod visible {} }
pub use source::*;
struct Item;
fn main() {}
