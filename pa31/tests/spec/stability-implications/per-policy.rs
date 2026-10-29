#![feature(staged_api)]
#![stable(feature="implication_root",since="1.0.0")]
#[stable(feature="parent",since="1.0.0")] pub fn parent(){}
#[stable(feature="other",since="1.0.0")]pub fn other(){}
#[unstable(feature="child",issue="none",implied_by="parent")] pub fn child()->u8{7}
#[unstable(feature="child",issue="none",implied_by="other")] pub fn sibling()->u8{7}
