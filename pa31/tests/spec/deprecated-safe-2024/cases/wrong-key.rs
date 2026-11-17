#![feature(rustc_attrs)]
#[rustc_deprecated_safe_2024(reason = "wrong key")]
unsafe fn marked() {}
