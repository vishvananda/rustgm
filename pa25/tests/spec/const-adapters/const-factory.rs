#![feature(const_closures, const_trait_impl, const_destruct)]
fn check(v:bool){if !v{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct Wrap<T>(T);impl<T>Wrap<T>{const fn one<A,F:FnMut(A)->T>(mut f:F)->impl FnMut(A)->Self{move|a|Self(f(a))}}
fn main(){let mut f=Wrap::<u8>::one(|x:u8|x+1);check(f(22).0==23);}
