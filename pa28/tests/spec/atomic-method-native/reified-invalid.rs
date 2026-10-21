use core::sync::atomic::{AtomicU32,Ordering};static A:AtomicU32=AtomicU32::new(0);fn main(){let get:fn(&AtomicU32,Ordering)->u32=AtomicU32::load;let _=get(&A,Ordering::Release);}
