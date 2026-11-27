#![allow(dead_code, unused_imports)]
#![feature(decl_macro)]
mod source { pub macro env() {} pub macro panic() {} pub mod panic {} }
mod hidden {
 mod env {}
 #[expect(hidden_glob_reexports)] mod panic {}
 pub use super::source::*;
}
pub use hidden::{env,panic};
fn main() {}
