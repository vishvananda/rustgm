#![feature(rustc_attrs)]
#![deny(deprecated_safe_2024)]
#[rustc_deprecated_safe_2024(audit_that = "the call meets the safety precondition")]
unsafe fn marked() {}
pub fn call() { marked(); }
