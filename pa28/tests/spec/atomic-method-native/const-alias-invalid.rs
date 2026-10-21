use core::sync::atomic::{AtomicU32,Ordering};static A:AtomicU32=AtomicU32::new(0);const O:Ordering=Ordering::Acquire;fn main(){A.store(7,O);}
