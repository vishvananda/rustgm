#![feature(decl_macro)]
struct S;
trait T { const VALUE: u8; }
macro install($s:ty) { impl T for $s { const VALUE: u8 = 7; } }
install!(S);
fn main() { println!("{}", <S as T>::VALUE); }
