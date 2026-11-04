#![feature(staged_api)]
#![unstable(feature="field_root",issue="none")]
macro_rules!make{($t:ty)=>{#[stable(feature="aggregate_api",since="1.0.0")]pub struct S(#[stable(feature="field_api",since="1.0.0")]pub $t);}}make!(u8);fn main(){let _=S(7).0;}
