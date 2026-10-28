#![feature(staged_api,rustc_attrs)]
#![stable(feature="feature_bound_client",since="1.0.0")]
extern crate feature_bound_provider as dep;
#[unstable(feature="feature_bound_client_fn",issue="none")]
#[unstable_feature_bound(feature_bound_probe,feature_bound_caller)]
pub fn call()->u32{dep::observed()}
