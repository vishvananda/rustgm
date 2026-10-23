#![feature(staged_api)]
#![stable(feature="fixture_root",since="1.0.0")]
mod hidden{pub fn answer()->u8{7}}
#[stable(feature="fixture_api",since="1.0.0")]
pub use hidden::answer;
