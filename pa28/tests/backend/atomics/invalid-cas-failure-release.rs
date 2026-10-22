use std::sync::atomic::{AtomicI32,Ordering::*,fence,compiler_fence};fn main(){let a=AtomicI32::new(0);let _=a.compare_exchange(0,1,SeqCst,Release);}
