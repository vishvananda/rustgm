use core::sync::atomic::{AtomicU32,Ordering};const X:u32={let a=AtomicU32::new(7);let f:fn(&AtomicU32,u32,Ordering)->u32=AtomicU32::fetch_max;f(&a,37,Ordering::Relaxed)};fn main(){let _=X;}
