#![feature(staged_api)]
#![unstable(feature="field_root",issue="none")]
#[stable(feature="aggregate_api",since="1.0.0")]pub struct S<T>(#[stable(feature="field_api",since="1.0.0")]pub T);fn main(){let s=S(7u8);let _=s.0;}
