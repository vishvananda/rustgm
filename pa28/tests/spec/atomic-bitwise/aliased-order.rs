use core::sync::atomic::{AtomicU32 as A,Ordering as O};const ORDER:O=O::Release;fn main(){let a=A::new(1);let _=a.fetch_xor(2,ORDER);let _=a.fetch_xor(1,ORDER);}
