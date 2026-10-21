use core::sync::atomic::{AtomicU32,Ordering};#[deny(invalid_atomic_ordering)]fn f(a:&AtomicU32){let _=||a.load(Ordering::Release);}
