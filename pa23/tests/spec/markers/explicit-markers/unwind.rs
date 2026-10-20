struct W<'a>(&'a mut u8);impl<'a> core::panic::UnwindSafe for W<'a>{}fn need<T:core::panic::UnwindSafe>(){}fn f<'a>(){need::<W<'a>>();}
fn main(){}
