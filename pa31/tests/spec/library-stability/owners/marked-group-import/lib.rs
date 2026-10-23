#![feature(staged_api)]
#![stable(feature="fixture_root",since="1.0.0")]
mod hidden{#[stable(feature="fixture_api",since="1.0.0")]
pub fn a()->u8{1}
#[stable(feature="fixture_api",since="1.0.0")]
pub fn b()->u8{2}}
#[stable(feature="fixture_api",since="1.0.0")]
pub use hidden::{a,b};
