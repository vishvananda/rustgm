#![feature(must_not_suspend)]
#[must_not_suspend]pub trait T{fn value(&self)->u32;}pub struct S;impl T for S{fn value(&self)->u32{7}}pub fn observe()->u32{let x:&dyn T=&S;x.value()}
