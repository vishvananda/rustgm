#![feature(impl_restriction)]
pub mod owner {
 pub impl(self) trait Restricted {}
 impl Restricted for u8 {}
 pub mod child { impl super::Restricted for u16 {} }
}
pub fn outside_use<T: owner::Restricted>() {}
