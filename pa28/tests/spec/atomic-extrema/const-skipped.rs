use core::sync::atomic::{AtomicU32,Ordering};const fn f(a:&AtomicU32){if false{let _=a.fetch_min(3,Ordering::Relaxed);let _=a.fetch_max(37,Ordering::SeqCst);}}fn main(){}
