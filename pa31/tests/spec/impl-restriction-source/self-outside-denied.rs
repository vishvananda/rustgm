#![feature(impl_restriction)]
pub mod owner { pub impl(self) trait Restricted {} }
impl owner::Restricted for u8 {}
