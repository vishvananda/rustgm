#![feature(impl_restriction)]
pub mod owner { pub impl(crate) trait Restricted {} }
impl owner::Restricted for u8 {}
pub mod sibling { impl crate::owner::Restricted for u16 {} }
