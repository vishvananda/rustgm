#![feature(staged_api)]
#![stable(feature="fixture_root",since="1.0.0")]
mod hidden{#[stable(feature="fixture_api",since="1.0.0")]
pub struct Hidden;}
#[stable(feature="fixture_api",since="1.0.0")]
pub fn answer()->hidden::Hidden{hidden::Hidden}
