#![feature(staged_api)]
#![stable(feature="fixture_root",since="1.0.0")]
#[unstable(feature="fixture_private",issue="none")]
pub fn experimental()->u8{7}
#[stable(feature="fixture_root",since="1.0.0")]
pub fn answer()->u8{experimental()}
