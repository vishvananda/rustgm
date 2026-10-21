use core::sync::atomic::{AtomicU32,Ordering};const X:u32={let a=AtomicU32::new(7);let f=AtomicU32::fetch_min;f(&a,3,Ordering::Relaxed)};fn main(){let _=X;}
