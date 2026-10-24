#![feature(decl_macro)]
struct S;
trait T { fn value() -> u8; }
macro first() { fn value() -> u8 { 7 } }
impl T for S { first!(); fn value() -> u8 { 9 } }
fn main() {}
