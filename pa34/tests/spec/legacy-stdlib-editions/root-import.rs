#![allow(dead_code,unused_imports)]
mod types { pub struct Byte; }
mod nested { use types::Byte; pub fn take(_: Byte) {} }
fn main() {}
