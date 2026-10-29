#![feature(staged_api)]
#![stable(feature="implication_root",since="1.0.0")]
#[unstable(feature="a",issue="none",implied_by="b")]pub fn a(){}
#[unstable(feature="b",issue="none",implied_by="a")]pub fn b(){}
