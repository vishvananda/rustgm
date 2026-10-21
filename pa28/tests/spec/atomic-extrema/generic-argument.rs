use core::sync::atomic::{AtomicU32,Ordering};fn main(){let a=AtomicU32::new(7);let _=a.fetch_max::<u32>(37,Ordering::Relaxed);}
