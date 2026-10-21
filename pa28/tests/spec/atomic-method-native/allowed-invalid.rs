use core::sync::atomic::{AtomicU32,Ordering};static A:AtomicU32=AtomicU32::new(0);#[allow(invalid_atomic_ordering)]fn main(){let _=A.load(Ordering::Release);}
