use core::sync::atomic::{AtomicU32 as A,Ordering as O};const X:u32={let a=A::new(1);if false{a.fetch_add(2,O::Relaxed)}else{7}};fn main(){let _=X;}
