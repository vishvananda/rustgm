#![feature(must_not_suspend)]
#[must_not_suspend]pub union S{pub v:u32}pub fn observe(x:S)->u32{x.v}
