use core::sync::atomic::{AtomicU32,Ordering};fn main(){let a=AtomicU32::new(7);let _=a.fetch_max(37,0u8);}
