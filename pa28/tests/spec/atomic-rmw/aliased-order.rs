use core::sync::atomic::{AtomicU32 as A,Ordering as O};const ORDER:O=O::Release;fn main(){let a=A::new(1);let _=a.swap(2,ORDER);let _=a.fetch_add(1,ORDER);}
