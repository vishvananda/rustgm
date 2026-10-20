fn need<T:core::panic::UnwindSafe>(){}fn f<T:core::panic::RefUnwindSafe>(){need::<*const T>();}
fn main(){}
