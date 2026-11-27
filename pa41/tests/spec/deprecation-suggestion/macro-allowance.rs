#![feature(allow_internal_unstable)]
#[allow_internal_unstable(deprecated_suggestion)]
macro_rules! make{()=>{#[deprecated(since = "1.96.0", note = "renamed to `as_nonzero_usize`", suggestion = "as_nonzero_usize")]fn old(){}}}
make!();fn main(){}
