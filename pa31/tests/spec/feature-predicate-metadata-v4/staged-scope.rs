#![feature(no_core,staged_api,rustc_attrs,predicate_api)]
#![no_core]
#![stable(feature="predicate_client",since="1.0.0")]
extern crate base;
#[unstable(feature="predicate_client_fn",issue="none")]
#[unstable_feature_bound(scope_only)]
pub fn answer()->u8{base::scoped(7u8)}
