#![feature(allow_internal_unstable)]
#[allow_internal_unstable(rustc_attrs)]macro_rules! make{()=>{#[rustc_skip_during_method_dispatch(array,boxed_slice)]trait Pick{}}}make!();
