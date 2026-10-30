#![feature(must_not_suspend)]
#[must_not_suspend]pub struct S(pub u32);pub fn observe()->u32{S(7).0}
