#![feature(staged_api)]
#![stable(feature="fixture_root",since="1.0.0")]
#[unstable(feature="fixture_private",issue="none")]
mod hidden{#[stable(feature="fixture_api",since="1.0.0")]
pub fn answer()->u8{7}}
fn main(){assert!(hidden::answer()==7);}
