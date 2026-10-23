#![feature(staged_api)]
#![stable(feature="fixture_root",since="1.0.0")]
#[stable(feature="fixture_root",since="1.0.0")]
#[rustc_const_unstable(feature="fixture_const",issue="none")]pub const fn later()->u8{7}
#[stable(feature="fixture_root",since="1.0.0")]
pub fn answer()->u8{later()}
