#![feature(rustc_attrs)]
pub struct S;impl S{pub fn parent(){#[rustc_confusables("swap")]fn nested()->u32{7}let _=nested();}}
