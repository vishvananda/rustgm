#![feature(rustc_attrs)]
#[rustc_deprecated_safe_2024(audit_that = 1)]
unsafe fn marked() {}
