#![allow(dead_code,unused_unsafe,internal_features)]
#[target_feature(enable="sse2")] unsafe fn f(){} #[target_feature(enable="sse2")]pub fn g(){f()}
