use core::sync::atomic::AtomicU32;struct W{a:u32,b:AtomicU32}static S:W=W{a:37,b:AtomicU32::new(7)};const X:u32=unsafe{core::ptr::read(&raw const S.a)};fn main(){let _=X;}
