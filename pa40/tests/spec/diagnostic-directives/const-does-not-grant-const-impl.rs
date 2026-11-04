#![allow(internal_features,unused_features,dead_code,unused_macros,unexpected_cfgs)]
#![feature(diagnostic_on_const,const_trait_impl)]
const trait T{fn f()->u8;}#[diagnostic::on_const(message="not const")]impl T for u8{fn f()->u8{7}}const V:u8=<u8 as T>::f();
