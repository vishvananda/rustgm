use core::sync::atomic::AtomicU32;static S:AtomicU32=AtomicU32::new(7);static P:&AtomicU32=&S;const R:&AtomicU32=unsafe{core::ptr::read(&raw const P)};fn main(){let _=R;}
