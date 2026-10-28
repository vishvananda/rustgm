#![feature(rustc_attrs)]
#![allow(internal_features,dead_code)]
#[rustc_as_ptr]
fn numeric()->u8{7}
trait Owner { #[rustc_as_ptr] fn marker(&self)->*const u8; }
struct Actual;
impl Actual { #[rustc_as_ptr] fn marker(&self)->*const u8 { self as *const Self as *const u8 } }
