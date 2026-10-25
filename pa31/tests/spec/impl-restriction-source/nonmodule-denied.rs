#![feature(impl_restriction)]
pub struct NotModule;
pub impl(in crate::NotModule) trait Restricted {}
