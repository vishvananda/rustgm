#![feature(staged_api)]
#![stable(feature="fixture_root",since="1.0.0")]
#[unstable(feature="fixture_private",issue="none")]
#[path="shared.rs"]pub mod a;
#[unstable(feature="fixture_other",issue="none")]
#[path="shared.rs"]pub mod b;
