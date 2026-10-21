use core::sync::atomic::{AtomicU32,Ordering};#[deny(invalid_atomic_ordering)]fn f(){#[allow(invalid_atomic_ordering)]fn g(a:&AtomicU32)->u32{a.load(Ordering::Release)}}
