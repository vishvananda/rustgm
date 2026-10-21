use core::sync::atomic::{AtomicU32,Ordering};static A:AtomicU32=AtomicU32::new(0);fn main(){let _=AtomicU32::load(&A,Ordering::Release);}
