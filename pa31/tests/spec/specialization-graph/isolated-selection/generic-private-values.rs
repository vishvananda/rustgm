#![allow(dead_code,unused_imports,incomplete_features,internal_features,unused_macros)]
#![feature(min_specialization)]
fn private<P,Q>(p:P,q:Q)->(P,Q){(p,q)}
trait V<A,B>{fn f<X>(a:A,b:B,x:X)->(A,B,X,u8);fn g()->u8;}
impl<P,Q> V<P,Q> for (Q,P){default fn f<X>(a:P,b:Q,x:X)->(P,Q,X,u8){let p=private::<Q,P>(b,a);(p.1,p.0,x,7)}default fn g()->u8{8}}
impl<R> V<R,u8> for (u8,R){fn g()->u8{9}}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn invoke<S:V<u16,u8>>()->u8{let p=S::f::<u16>;let x=p(31u16,5u8,11u16);check(x.0==31);check(x.1==5);check(x.2==11);x.3+S::g()}
fn main(){let p=<(u8,u16) as V<u16,u8>>::f::<u32>;let x=p(17u16,3u8,99u32);check(x.0==17);check(x.1==3);check(x.2==99);check(x.3==7);check(invoke::<(u8,u16)>()==16);}
