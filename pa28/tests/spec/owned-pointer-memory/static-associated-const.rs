use core::sync::atomic::AtomicU32;static S:AtomicU32=AtomicU32::new(7);struct W;impl W{const A:AtomicU32=unsafe{core::ptr::read(&raw const S)};}static X:AtomicU32=W::A;fn main(){let _=&X;}
