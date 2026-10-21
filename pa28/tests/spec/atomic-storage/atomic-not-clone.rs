use core::sync::atomic::AtomicU32;
fn check<T:Clone>(){}fn f(){check::<AtomicU32>();}
