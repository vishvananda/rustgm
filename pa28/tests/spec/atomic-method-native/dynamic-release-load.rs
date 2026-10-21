use core::sync::atomic::{AtomicU32,Ordering};static A:AtomicU32=AtomicU32::new(0);fn read(o:Ordering){let _=A.load(o);}fn main(){read(Ordering::Release);}
