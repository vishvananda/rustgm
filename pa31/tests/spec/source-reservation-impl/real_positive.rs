#![feature(rustc_attrs)]
trait A{fn n()->u32;}struct S;
#[rustc_reservation_impl="future impl"]impl<T> A for T{fn n()->u32{99}}impl A for S{fn n()->u32{7}}#[unsafe(no_mangle)]pub extern "C" fn observed()->u32{<S as A>::n()}
