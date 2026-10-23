#![feature(staged_api)]
#![stable(feature="fixture_root",since="1.0.0")]
#[unstable(feature="fixture_private",issue="none")]
pub mod parent{#[stable(feature="fixture_api",since="1.0.0")]
pub mod child{pub fn answer()->u8{7}}}
