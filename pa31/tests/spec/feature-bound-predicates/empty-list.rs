#![feature(staged_api, rustc_attrs)]
#![stable(feature="feature_bound_fixture", since="1.0.0")]
#[unstable(feature="feature_bound_caller", issue="none")]
#[unstable_feature_bound()]
pub fn declaration() -> u32 { 7 }
