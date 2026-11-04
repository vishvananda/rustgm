#![feature(staged_api)]
#![unstable(feature="field_root",issue="none")]
#[stable(feature="aggregate_api",since="1.0.0")]pub struct S(#[stable(feature="field_api",since="1.0.0")]pub u8,#[unstable(feature="field_next",issue="none")]pub u8);fn main(){let S(a,b)=S(7,9);let _=a+b;}
