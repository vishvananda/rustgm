use core::sync::atomic::{AtomicU32,Ordering};fn main(){let a=AtomicU32::new(7);let _=AtomicU32::fetch_min(&a,3,Ordering::SeqCst);let _=AtomicU32::fetch_max(&a,37,Ordering::SeqCst);}
