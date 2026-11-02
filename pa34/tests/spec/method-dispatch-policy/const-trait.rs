#![feature(rustc_attrs)]
#![feature(const_trait_impl)]
#[rustc_skip_during_method_dispatch(boxed_slice)]const trait Pick{}
