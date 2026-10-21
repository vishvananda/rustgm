use core::sync::atomic::{AtomicU32,Ordering};fn f(a:&AtomicU32)->u32{let f:fn(&AtomicU32,Ordering)->u32=AtomicU32::load;f(a,Ordering::Release)}
