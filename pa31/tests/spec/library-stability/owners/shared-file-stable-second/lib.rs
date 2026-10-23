#![feature(staged_api)]
#![stable(feature="fixture_root",since="1.0.0")]
#[unstable(feature="fixture_private",issue="none")]
#[path="shared.rs"]pub mod a;
#[stable(feature="fixture_api",since="1.0.0")]
#[path="shared.rs"]pub mod b;
