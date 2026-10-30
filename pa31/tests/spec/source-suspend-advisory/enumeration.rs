#![feature(must_not_suspend)]
#[must_not_suspend]pub enum S{V(u32)}pub fn observe()->u32{match S::V(7){S::V(v)=>v}}
