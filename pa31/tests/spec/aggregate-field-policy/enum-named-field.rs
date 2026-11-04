#![feature(staged_api)]
#![unstable(feature="field_root",issue="none")]
#[stable(feature="aggregate_api",since="1.0.0")]pub enum E{#[stable(feature="aggregate_api",since="1.0.0")]V{#[unstable(feature="field_next",issue="none")]x:u8}}fn main(){let E::V{x}=E::V{x:7};let _=x;}
