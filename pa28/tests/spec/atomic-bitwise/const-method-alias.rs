use core::sync::atomic::{AtomicU32 as A,Ordering as O};const X:u32={let a=A::new(1);let f=A::fetch_or;f(&a,2,O::Relaxed)};fn main(){let _=X;}
