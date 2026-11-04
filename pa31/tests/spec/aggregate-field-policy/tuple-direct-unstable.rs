#![feature(staged_api)]
#![unstable(feature="field_root",issue="none")]
#[stable(feature="aggregate_api",since="1.0.0")]pub struct S(#[unstable(feature="field_next",issue="none")]pub u8);fn main(){let _=S(7).0;}
