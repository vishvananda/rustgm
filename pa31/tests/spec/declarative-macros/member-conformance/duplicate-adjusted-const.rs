#![feature(decl_macro)]
struct S;
trait T { const VALUE:u8; }
macro first() { const VALUE:u8=7; }
impl T for S { first!(); const VALUE:u8=9; }
fn main() {}
