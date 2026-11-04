#![feature(staged_api)]
#![unstable(feature="field_root",issue="none")]
#[stable(feature="aggregate_api",since="1.0.0")]pub enum E{V(#[stable(feature="field_api",since="1.0.0")]u8)}fn main(){let E::V(x)=E::V(7);let _=x;}
