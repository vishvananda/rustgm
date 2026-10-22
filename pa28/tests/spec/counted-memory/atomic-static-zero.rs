use core::sync::atomic::AtomicU32;static A:AtomicU32=AtomicU32::new(7);const N:()={let mut b=AtomicU32::new(0);unsafe{core::ptr::copy_nonoverlapping(&raw const A,&raw mut b,0);}};fn main(){let _=N;}
