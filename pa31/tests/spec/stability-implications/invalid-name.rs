#![feature(staged_api)]
#![stable(feature="implication_root",since="1.0.0")]
#[stable(feature="parent",since="1.0.0")] pub fn parent(){}
#[unstable(feature="child",issue="none",implied_by="not-a-feature")] pub fn child()->u8{7}
