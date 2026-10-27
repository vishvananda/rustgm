#![allow(dead_code,unused_unsafe,internal_features)]
#[target_feature(enable="sse2")] fn f(){} #[target_feature(enable="sse")]pub fn g(){f()}
