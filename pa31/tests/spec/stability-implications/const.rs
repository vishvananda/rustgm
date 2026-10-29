#![feature(staged_api)]
#![stable(feature="implication_root",since="1.0.0")]
#[stable(feature="parent",since="1.0.0")] pub fn parent(){}
#[stable(feature="const_api",since="1.0.0")]
#[rustc_const_unstable(feature="const_child",issue="none",implied_by="parent")]pub const fn value()->u8{7}
