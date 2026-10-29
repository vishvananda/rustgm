#![feature(staged_api)]
#![stable(feature="implication_root",since="1.0.0")]
#[unstable(feature="child",issue="none",implied_by="child")] pub fn child()->u8{7}
