#![feature(allow_internal_unstable)]
#[allow_internal_unstable(impl_restriction)]
macro_rules! define { () => { pub impl(crate) trait Restricted {} }; }
define!();
