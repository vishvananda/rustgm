use core::sync::atomic::AtomicU32;enum E{Plain(u32),Atomic(AtomicU32)}static S:E=E::Plain(37);const X:E=unsafe{core::ptr::read(&raw const S)};fn main(){let _=X;}
