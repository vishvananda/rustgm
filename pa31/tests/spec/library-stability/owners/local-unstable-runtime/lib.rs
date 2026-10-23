#![feature(staged_api)]
#![stable(feature="fixture_root",since="1.0.0")]
#[unstable(feature="fixture_private",issue="none")]
fn answer()->u8{7}
fn main(){assert!(answer()==7);}
