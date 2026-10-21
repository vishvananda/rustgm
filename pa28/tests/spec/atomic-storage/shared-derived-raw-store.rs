#![allow(invalid_reference_casting)]
use core::sync::atomic::AtomicU32;
const X:u32={let x=AtomicU32::new(7);let p=(&x) as *const AtomicU32 as *mut AtomicU32;unsafe{*p=AtomicU32::new(9);}0};fn main(){let _=X;}
