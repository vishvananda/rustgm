#![feature(no_core, const_trait_impl, const_policy_runtime)]
#![no_core]
extern crate base;

pub const VALUE: u8 = <base::Mark as base::Evaluation>::runtime();
