use core::sync::atomic::{AtomicU32,Ordering};#[forbid(invalid_atomic_ordering)]mod m{#[allow(invalid_atomic_ordering)]fn f(a:&super::AtomicU32)->u32{a.load(super::Ordering::Release)}}
