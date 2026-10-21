use core::sync::atomic::AtomicU32;
fn check<T:Send+Sync+Unpin+core::panic::UnwindSafe+core::panic::RefUnwindSafe>(){}fn f(){check::<AtomicU32>();check::<[AtomicU32;0]>();}
