use core::sync::atomic::AtomicU32;
fn check<T:Copy>(){}fn f(){check::<AtomicU32>();}
