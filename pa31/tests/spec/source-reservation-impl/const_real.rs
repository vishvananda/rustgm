#![feature(rustc_attrs)]
#![feature(const_trait_impl)]const trait A{fn n()->u32;}struct S;#[rustc_reservation_impl="future impl"]const impl<T>A for T{fn n()->u32{99}}const impl A for S{fn n()->u32{7}}const V:u32=<S as A>::n();#[unsafe(no_mangle)]pub extern "C" fn observed()->u32{V}
