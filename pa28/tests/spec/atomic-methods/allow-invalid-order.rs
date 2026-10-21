use core::sync::atomic::{AtomicU32,Ordering};#[allow(invalid_atomic_ordering)]fn f(a:&AtomicU32)->u32{a.load(Ordering::Release)}
