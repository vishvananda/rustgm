#![feature(decl_macro)]
struct S;
macro declare($t:ident) { trait $t { fn value() -> u8; } }
declare!(T);
impl T for S { fn value() -> u8 { 7 } }
fn main() {}
