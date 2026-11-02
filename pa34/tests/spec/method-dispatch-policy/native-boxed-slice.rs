#![feature(rustc_attrs)]
#[rustc_skip_during_method_dispatch(boxed_slice)]trait Pick{fn pick(&self)->u64;}impl Pick for Box<[u8]>{fn pick(&self)->u64{7}}impl Pick for [u8]{fn pick(&self)->u64{9}}fn main(){let x:Box<[u8]>=Box::new([0u8]);println!("{}",x.pick());}
