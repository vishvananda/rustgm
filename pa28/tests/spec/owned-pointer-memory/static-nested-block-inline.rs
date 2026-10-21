use core::sync::atomic::AtomicU32;static S:AtomicU32=AtomicU32::new(7);static X:AtomicU32={const{unsafe{core::ptr::read(&raw const S)}}};fn main(){let _=&X;}
