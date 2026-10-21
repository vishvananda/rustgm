use core::sync::atomic::AtomicU32;struct W{a:(),b:AtomicU32}static S:W=W{a:(),b:AtomicU32::new(7)};const X:()=unsafe{core::ptr::read(&raw const S.a)};fn main(){let _=X;}
