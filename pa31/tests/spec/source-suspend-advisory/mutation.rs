#![feature(must_not_suspend)]
#[must_not_suspend]pub struct S{v:u32}pub fn observe()->u32{let mut x=S{v:7};x.v=9;x.v}
