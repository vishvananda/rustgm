#![feature(decl_macro)]
struct S;
trait T { type Output; }
macro first() { type Output=u8; }
impl T for S { first!(); type Output=u16; }
fn main() {}
