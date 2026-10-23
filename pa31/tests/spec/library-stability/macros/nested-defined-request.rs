#![feature(staged_api)]
#![stable(feature="fixture_root",since="1.0.0")]
#[stable(feature="fixture_root",since="1.0.0")]
pub mod child{#![feature(fixture_private)]
#[unstable(feature="fixture_private",issue="none")]
pub fn answer()->u8{7}}
