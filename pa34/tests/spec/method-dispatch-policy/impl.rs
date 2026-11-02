#![feature(rustc_attrs)]
trait Pick{}#[rustc_skip_during_method_dispatch(array)]impl Pick for u8{}
