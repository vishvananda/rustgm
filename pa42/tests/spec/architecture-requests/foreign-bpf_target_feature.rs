#![feature(bpf_target_feature)]
#[target_feature(enable="alu32")]unsafe fn probe(){}fn main(){}
