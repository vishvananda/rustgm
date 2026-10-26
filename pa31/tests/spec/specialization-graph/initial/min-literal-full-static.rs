#![allow(dead_code,unused_imports,incomplete_features,internal_features,unused_macros)]
#![feature(min_specialization,allow_internal_unstable)]
#[allow_internal_unstable(specialization)]macro_rules! family{()=>{trait V{fn f()->u8;}impl<T>V for T{default fn f()->u8{1}}impl<T:'static>V for &'static T{fn f()->u8{2}}}}family!();fn main(){}
