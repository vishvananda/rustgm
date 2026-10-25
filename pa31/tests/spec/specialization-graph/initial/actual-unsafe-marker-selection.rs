#![allow(dead_code,unused_imports,incomplete_features,internal_features,unused_macros)]
#![feature(min_specialization,rustc_attrs)]
#[rustc_unsafe_specialization_marker]trait Fast:Clone{}impl Fast for u8{}trait V:Clone{fn f()->u8;}impl<T:Clone>V for T{default fn f()->u8{1}}impl<T:Fast>V for T{fn f()->u8{2}}fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){check(<u8 as V>::f()==2);check(<u16 as V>::f()==1);}
