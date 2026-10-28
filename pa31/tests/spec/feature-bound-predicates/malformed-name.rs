#![feature(staged_api, rustc_attrs)]
#![stable(feature="feature_bound_fixture", since="1.0.0")]
#[unstable(feature="feature_bound_probe", issue="none")]
#[unstable_feature_bound("feature_bound_probe")]
pub trait Wrong {}
