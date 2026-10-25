#![allow(dead_code,unused_imports,incomplete_features,internal_features)]
#![feature(allow_internal_unstable)]
trait T{fn f()->u8;}
#[allow_internal_unstable(specialization)]macro_rules! m{($i:item)=>{$i}}m!(impl<X>T for X{default fn f()->u8{1}}
);fn main(){}
