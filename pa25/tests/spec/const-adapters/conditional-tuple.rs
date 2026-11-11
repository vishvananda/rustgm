#![feature(const_closures, const_trait_impl, const_destruct)]
fn check(v:bool){if !v{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
const fn one<T,A,F>(mut f:F)->impl [const]FnMut(A)->(T,)+[const]core::marker::Destruct where F:[const]FnMut(A)->T+[const]core::marker::Destruct{const move|a|(f(a),)}
fn main(){let mut f=one(|x:u8|x+1);check(f(22).0==23);}
