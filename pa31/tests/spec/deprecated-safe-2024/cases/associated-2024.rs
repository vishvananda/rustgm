#![feature(rustc_attrs)]
struct Owner;
impl Owner { #[rustc_deprecated_safe_2024(audit_that = "the call meets the safety precondition")] unsafe fn method(&self) {} }
pub fn call() { Owner.method(); }
