use core::sync::atomic::{AtomicU32,Ordering};#[warn(invalid_atomic_ordering)]fn f(a:&AtomicU32)->u32{a.load(Ordering::Release)}fn main(){}
