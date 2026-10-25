#![feature(allow_internal_unstable)]
#[allow_internal_unstable(impl_restriction)]
macro_rules! relay { ($item:item) => { $item }; }
relay! { pub impl(crate) trait Restricted {} }
