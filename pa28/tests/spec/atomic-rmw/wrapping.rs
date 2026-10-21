use core::sync::atomic::{AtomicU32 as A,Ordering as O};fn main(){let a=A::new(4294967295);let _=a.fetch_add(1,O::Relaxed);let _=a.fetch_add(4294967295,O::SeqCst);}
