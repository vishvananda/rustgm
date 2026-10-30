#![feature(must_not_suspend)]
#[must_not_suspend]pub struct S(u32);pub const V:u32=S(7).0;
