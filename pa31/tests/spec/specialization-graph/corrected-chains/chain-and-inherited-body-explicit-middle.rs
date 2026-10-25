#![allow(dead_code,unused_imports,incomplete_features,internal_features,unused_macros)]
#![feature(min_specialization)]
trait V{fn f()->u8;fn g()->u8;}impl<T> V for T{default fn f()->u8{1}default fn g()->u8{2}}impl<T> V for [T;1]{default fn f()->u8{1}default fn g()->u8{3}}impl V for [u8;1]{fn f()->u8{4}}fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn invoke<T:V>()->u8{T::f()*10+T::g()}fn main(){check(invoke::<[u8;1]>()==43);check(invoke::<[u16;1]>()==13);check(invoke::<u16>()==12);let p= <[u8;1] as V>::g;check(p()==3);}
