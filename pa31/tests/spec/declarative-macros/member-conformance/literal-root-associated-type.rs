#![feature(decl_macro)]
struct S;
trait T { type Output; fn value() -> Self::Output; }
macro install($s:ty) { impl T for $s { type Output=u8; fn value() -> u8 { 7 } } }
install!(S);
fn main() { println!("{}", <S as T>::value()); }
