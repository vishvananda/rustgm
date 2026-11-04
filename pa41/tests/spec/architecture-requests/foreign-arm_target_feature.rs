#![feature(arm_target_feature)]
#[target_feature(enable="neon")]unsafe fn probe(){}fn main(){}
