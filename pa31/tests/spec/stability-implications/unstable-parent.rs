#![feature(staged_api)]
#![stable(feature="implication_root",since="1.0.0")]
#[unstable(feature="parent",issue="none")] pub fn parent(){}
#[unstable(feature="child",issue="none",implied_by="parent")] pub fn child()->u8{7}
