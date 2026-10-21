use core::sync::atomic::{AtomicU32 as A,Ordering as O};const X:u32={let a=A::new(1);a.fetch_sub(2,O::Relaxed)};fn main(){let _=X;}
