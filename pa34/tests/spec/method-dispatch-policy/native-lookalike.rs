#![feature(rustc_attrs)]
struct Box([u8;1]);#[rustc_skip_during_method_dispatch(array,boxed_slice)]trait Pick{fn pick(&self)->u64;}impl Pick for Box{fn pick(&self)->u64{7}}fn main(){println!("{}",Box([0u8]).pick());}
