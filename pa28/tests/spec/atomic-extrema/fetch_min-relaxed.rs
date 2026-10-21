use core::sync::atomic::{AtomicU32,Ordering};fn main(){let a=AtomicU32::new(2147483648);let _=a.fetch_min(37,Ordering::Relaxed);}
