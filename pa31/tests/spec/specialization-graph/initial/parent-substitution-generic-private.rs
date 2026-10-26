#![allow(dead_code,unused_imports,incomplete_features,internal_features,unused_macros)]
#![feature(min_specialization)]
fn private()->u8{7}trait V<A,B>{fn f<X>(x:X)->(u8,X);fn g()->u8;}
impl<P,Q> V<P,Q> for (Q,P){default fn f<X>(x:X)->(u8,X){(private()+std::mem::size_of::<P>() as u8+std::mem::size_of::<Q>() as u8,x)}default fn g()->u8{8}}
impl<R> V<R,u8> for (u8,R){fn g()->u8{9}}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn invoke<S:V<u16,u8>>()->u8{let p=S::f::<u16>;p(11).0+S::g()}fn main(){let p=<(u8,u16) as V<u16,u8>>::f::<u32>;let x=p(99);check(x.0==10);check(x.1==99);check(invoke::<(u8,u16)>()==19);}
