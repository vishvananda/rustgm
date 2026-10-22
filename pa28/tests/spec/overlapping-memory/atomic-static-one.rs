use core::sync::atomic::AtomicU32;static A:AtomicU32=AtomicU32::new(7);const N:()={let mut b=AtomicU32::new(0);unsafe{core::ptr::copy(&raw const A,&raw mut b,1);}};fn main(){let _=N;}
