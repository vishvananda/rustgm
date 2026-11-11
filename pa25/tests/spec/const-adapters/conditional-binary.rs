#![feature(const_closures, const_trait_impl, const_destruct)]
fn check(v:bool){if !v{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct Wrap<T>(T);impl<T>Wrap<T>{const fn two<A,B,F>(mut f:F)->impl [const]FnMut(A,B)->Self+[const]core::marker::Destruct where F:[const]FnMut(A,B)->T+[const]core::marker::Destruct{const move|a,b|Self(f(a,b))}}
fn main(){let mut f=Wrap::<u8>::two(|x:u8,y:u8|x+y);check(f(21,2).0==23);}
