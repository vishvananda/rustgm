#![allow(dead_code,unused_imports,incomplete_features,internal_features)]
#![deny(unstable_syntax_pre_expansion)]
trait T{fn f()->u8;}
#[allow(unstable_syntax_pre_expansion)]#[cfg(any())]impl<X>T for X{default fn f()->u8{1}}
fn main(){}
