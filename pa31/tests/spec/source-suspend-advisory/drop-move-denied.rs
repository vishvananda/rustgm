#![feature(must_not_suspend)]
#[must_not_suspend]pub struct S(u32);impl Drop for S{fn drop(&mut self){}}pub fn observe()->u32{let x=S(7);let y=x;x.0+y.0}
