#![feature(staged_api)]
#![stable(feature="fixture_root",since="1.0.0")]
#[stable(feature="fixture_api",since="1.0.0")]
pub mod child{fn hidden()->u8{7}}
