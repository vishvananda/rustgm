use core::sync::atomic::{AtomicU32,Ordering};fn order()->Ordering{Ordering::AcqRel}fn main(){let a=AtomicU32::new(7);let _=a.fetch_min(3,order());let _=a.fetch_max(37,order());}
