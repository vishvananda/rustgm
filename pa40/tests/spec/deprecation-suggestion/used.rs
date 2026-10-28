#![feature(deprecated_suggestion)]
#[deprecated(since = "1.96.0", note = "renamed to `as_nonzero_usize`", suggestion = "as_nonzero_usize")]
fn old()->usize{7}
fn main(){let _=old();}
