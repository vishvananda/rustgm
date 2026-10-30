#![feature(must_not_suspend)]
#[must_not_suspend]pub union S{pub v:u32}pub fn observe()->u32{unsafe{S{v:7}.v}}
