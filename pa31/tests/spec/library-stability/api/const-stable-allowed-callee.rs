#![feature(staged_api)]
#![stable(feature="fixture_root",since="1.0.0")]
#![feature(rustc_attrs)]
#[stable(feature="fixture_root",since="1.0.0")]
#[rustc_const_unstable(feature="fixture_const",issue="none")]pub const fn later()->u8{7}
#[stable(feature="fixture_root",since="1.0.0")]
#[rustc_const_stable(feature="fixture_root",since="1.0.0")]#[rustc_allow_const_fn_unstable(fixture_const)]pub const fn answer()->u8{later()}
