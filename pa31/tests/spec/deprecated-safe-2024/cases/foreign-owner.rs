#![feature(rustc_attrs)]
unsafe extern "C" { #[rustc_deprecated_safe_2024(audit_that = "the call meets the safety precondition")] fn wrong(); }
