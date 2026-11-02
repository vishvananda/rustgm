#![feature(rustc_attrs)]
struct Pick{#[rustc_skip_during_method_dispatch(array)]v:u8}
