#![feature(must_not_suspend)]
#[must_not_suspend]pub struct S{v:u32}pub fn change(x:&S){x.v=9;}
