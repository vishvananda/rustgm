#![feature(staged_api, rustc_attrs)]
#![stable(feature="feature_bound_fixture", since="1.0.0")]
#[unstable(feature="feature_bound_probe", issue="none")]
#[unstable_feature_bound(feature_bound_probe)]
pub trait NeedsGate { fn value(&self) -> u32; }
#[stable(feature="feature_bound_fixture", since="1.0.0")]
pub struct Value;
#[unstable(feature="feature_bound_probe", issue="none")]
#[unstable_feature_bound(feature_bound_probe)]
impl NeedsGate for Value { fn value(&self) -> u32 { 7 } }
#[unstable(feature="feature_bound_caller", issue="none")]
#[unstable_feature_bound(feature_bound_probe)]
#[unsafe(no_mangle)]
pub extern "C" fn observed() -> u32 { let invoke = || <Value as NeedsGate>::value(&Value); invoke() }
