use core::sync::atomic::AtomicU32;static S:AtomicU32=AtomicU32::new(7);static X:AtomicU32={const Y:AtomicU32=unsafe{core::ptr::read(&raw const S)};Y};fn main(){let _=&X;}
