#![feature(nvptx_target_feature)]
#[target_feature(enable="sm_80")]unsafe fn probe(){}fn main(){}
