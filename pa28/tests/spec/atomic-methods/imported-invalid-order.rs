use core::sync::atomic::{AtomicU32,Ordering};use core::sync::atomic::Ordering::Release as Bad;fn f(a:&AtomicU32)->u32{a.load(Bad)}
