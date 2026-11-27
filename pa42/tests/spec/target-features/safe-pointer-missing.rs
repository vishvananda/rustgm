#![allow(dead_code,unused_unsafe,internal_features)]
#[target_feature(enable="sse2")] fn f(){} pub fn g(){let _:fn()=f;}
