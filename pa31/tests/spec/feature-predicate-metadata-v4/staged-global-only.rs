#![feature(no_core,staged_api,rustc_attrs,predicate_api,predicate_probe)]
#![no_core]
#![stable(feature="predicate_client",since="1.0.0")]
extern crate base;
#[unstable(feature="predicate_client_fn",issue="none")]
pub fn answer()->u8{base::gated(7u8)}
