#![feature(s390x_target_feature)]
#[target_feature(enable="vector")]unsafe fn probe(){}fn main(){}
