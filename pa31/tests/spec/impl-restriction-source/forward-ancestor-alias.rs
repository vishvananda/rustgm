#![feature(impl_restriction)]
pub mod owner {
 pub mod child {
  pub impl(in ancestor) trait Restricted {}
  use crate::owner as ancestor;
 }
 impl child::Restricted for u8 {}
}
