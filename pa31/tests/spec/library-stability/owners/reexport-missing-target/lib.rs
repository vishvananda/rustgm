#![feature(staged_api)]
#![stable(feature="fixture_root",since="1.0.0")]
mod hidden{pub fn answer()->u8{7}}
pub use hidden::answer;
