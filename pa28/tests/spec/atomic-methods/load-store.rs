use core::sync::atomic::{AtomicU32,Ordering};fn f(a:&AtomicU32)->u32{a.store(7,Ordering::Release);a.load(Ordering::Acquire)}
