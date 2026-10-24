#![feature(decl_macro)]
struct S;
macro declare($t:ident,$member:ident) { trait $t { fn $member() -> u8; } }
declare!(T,value);
macro install($s:ty) { impl T for $s { fn value() -> u8 { 7 } } }
install!(S);
fn main() { println!("{}", <S as T>::value()); }
