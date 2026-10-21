use core::sync::atomic::{AtomicU32,Ordering};const fn f(a:&AtomicU32){if false{a.store(7,Ordering::Relaxed)}}
