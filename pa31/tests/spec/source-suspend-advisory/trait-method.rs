#![feature(must_not_suspend)]
pub trait T{#[must_not_suspend]fn value(&self)->u32;}
