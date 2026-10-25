#![feature(impl_restriction)]
pub mod owner {
 pub mod child { pub impl(super) trait Restricted {} }
 impl child::Restricted for u8 {}
 pub mod sibling { impl super::child::Restricted for u16 {} }
}
