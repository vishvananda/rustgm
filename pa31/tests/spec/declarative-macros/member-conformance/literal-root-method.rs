#![feature(decl_macro)]
struct S;
trait T { fn value() -> u8; }
macro install($s:ty) { impl T for $s { fn value() -> u8 { 7 } } }
install!(S);
fn main() { println!("{}", <S as T>::value()); }
