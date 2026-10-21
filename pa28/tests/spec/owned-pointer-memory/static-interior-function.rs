use core::sync::atomic::AtomicU32;static S:AtomicU32=AtomicU32::new(7);const fn get()->AtomicU32{unsafe{core::ptr::read(&raw const S)}}static X:AtomicU32=get();fn main(){let _=&X;}
