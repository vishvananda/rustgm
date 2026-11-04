#![feature(wasm_target_feature)]
#[target_feature(enable="simd128")]unsafe fn probe(){}fn main(){}
