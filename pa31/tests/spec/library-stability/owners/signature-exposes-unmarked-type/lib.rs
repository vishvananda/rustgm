#![feature(staged_api)]
#![stable(feature="fixture_root",since="1.0.0")]
struct Hidden;
#[stable(feature="fixture_api",since="1.0.0")]
pub fn answer()->Hidden{Hidden}
