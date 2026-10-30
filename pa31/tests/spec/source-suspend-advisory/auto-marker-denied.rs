#![feature(must_not_suspend)]
#[must_not_suspend]pub trait T{}pub fn add_send(x:&dyn T)->&(dyn T+Send){x}
