use core::sync::atomic::AtomicU32;static S:AtomicU32=AtomicU32::new(7);const X:AtomicU32=unsafe{core::ptr::read(&raw const S)};fn main(){let _=X;}
