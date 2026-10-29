#![feature(staged_api)]
#![stable(feature="implication_root",since="1.0.0")]
#[stable(feature="parent",since="1.0.0")] pub fn parent(){}
#[stable(feature="child",since="1.0.0",implied_by="parent")]pub fn child(){}
