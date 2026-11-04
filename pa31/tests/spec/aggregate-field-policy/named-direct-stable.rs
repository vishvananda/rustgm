#![feature(staged_api)]
#![unstable(feature="field_root",issue="none")]
#[stable(feature="aggregate_api",since="1.0.0")]pub struct S{#[stable(feature="field_api",since="1.0.0")]pub x:u8}fn main(){let S{x}=S{x:7};let _=x;}
