#![feature(staged_api)]
#![unstable(feature="field_root",issue="none")]
macro_rules!make{()=>{#[stable(feature="aggregate_api",since="1.0.0")]pub struct S(#[stable(feature="field_api",since="1.0.0")]pub u8);}}make!();fn main(){let _=S(7).0;}
