#![feature(staged_api,rustc_attrs,feature_bound_caller)]
#![stable(feature="feature_bound_client",since="1.0.0")]
extern crate feature_bound_provider as dep;
#[unstable(feature="feature_bound_client_fn",issue="none")]
#[unstable_feature_bound(feature_bound_probe)]
pub fn call()->u32{dep::observed()}
