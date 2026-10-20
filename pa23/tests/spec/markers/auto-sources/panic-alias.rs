use core::panic::UnwindSafe as Safe;fn need<T:Safe>(){}fn f(){need::<u8>();}
fn main(){}
