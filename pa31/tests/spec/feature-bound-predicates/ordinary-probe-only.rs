#![feature(feature_bound_probe)]
extern crate feature_bound_provider as dep;
pub fn call() -> u32 { dep::observed() }
