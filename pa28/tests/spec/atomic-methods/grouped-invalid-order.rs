use core::sync::atomic::{AtomicU32,Ordering};fn f(a:&AtomicU32)->u32{a.load(((Ordering::Release)))}
