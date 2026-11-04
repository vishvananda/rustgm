#![feature(loongarch_target_feature)]
#[target_feature(enable="lsx")]unsafe fn probe(){}fn main(){}
