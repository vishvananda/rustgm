#![feature(allow_internal_unsafe)]
#![allow(internal_features)]
pub unsafe fn target()->u8{7}
#[macro_export]#[allow_internal_unsafe]macro_rules! allowed{()=>{unsafe{$crate::target()}}}
#[macro_export]macro_rules! unmarked{()=>{unsafe{$crate::target()}}}
