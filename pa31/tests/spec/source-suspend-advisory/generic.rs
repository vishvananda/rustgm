#![feature(must_not_suspend)]
#[must_not_suspend]pub struct S<T>{v:T}pub fn observe()->u32{S{v:7u32}.v}
