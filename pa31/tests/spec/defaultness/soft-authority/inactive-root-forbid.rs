#![allow(dead_code,unused_imports,incomplete_features,internal_features)]
#![forbid(unstable_syntax_pre_expansion)]
trait T{fn f()->u8;}
#[cfg(any())]impl<X>T for X{default fn f()->u8{1}}
fn main(){}
