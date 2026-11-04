#![feature(staged_api)]
#![unstable(feature="field_root",issue="none")]
#[stable(feature="aggregate_api",since="1.0.0")]pub struct S{pub x:u8}fn main(){let _=S{x:7}.x;}
