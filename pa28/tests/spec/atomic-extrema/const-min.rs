use core::sync::atomic::{AtomicU32,Ordering};const X:u32={let a=AtomicU32::new(7);a.fetch_min(3,Ordering::Relaxed)};fn main(){let _=X;}
