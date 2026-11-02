#![feature(rustc_attrs)]
#[rustc_skip_during_method_dispatch(array)]trait Pick{fn pick(self)->u64;}impl Pick for [u8;1]{fn pick(self)->u64{7}}impl Pick for &[u8;1]{fn pick(self)->u64{9}}fn main(){println!("{}",Pick::pick([0u8]));}
