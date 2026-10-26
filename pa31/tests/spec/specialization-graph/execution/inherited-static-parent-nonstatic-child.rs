#![allow(dead_code,unused_imports,incomplete_features,internal_features,unused_macros)]
#![feature(specialization)]
struct W<T>(T);trait V{fn f()->u8;}impl<T:'static>V for W<T>{default fn f()->u8{1}}impl<'a>V for W<&'a u8>{}fn use_it<'a>(_: &'a u8)->u8{<W<&'a u8> as V>::f()}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let x=7;check(use_it(&x)==1);}
