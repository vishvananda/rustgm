#![feature(rustc_attrs)]
trait A{extern "C" fn observed()->u32;}struct S;#[rustc_reservation_impl="future impl"]impl A for S{#[unsafe(no_mangle)]extern "C" fn observed()->u32{7}}
