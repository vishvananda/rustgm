#![allow(dead_code,unused_imports,incomplete_features,internal_features,unused_macros)]
#![feature(allow_internal_unstable)]
trait V{fn f()->u8;}#[allow_internal_unstable(specialization)]macro_rules! family{()=>{impl<T>V for T{default fn f()->u8{1}}impl V for u8{fn f()->u8{2}}}}family!();fn main(){}
