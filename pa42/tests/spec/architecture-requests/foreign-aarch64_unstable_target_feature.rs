#![feature(aarch64_unstable_target_feature)]
#[target_feature(enable="sme")]unsafe fn probe(){}fn main(){}
