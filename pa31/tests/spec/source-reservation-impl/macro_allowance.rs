#![feature(allow_internal_unstable)]
trait A{}struct S;#[allow_internal_unstable(rustc_attrs)]macro_rules! make{()=>{#[rustc_reservation_impl="future impl"]impl A for S{}}}make!();
