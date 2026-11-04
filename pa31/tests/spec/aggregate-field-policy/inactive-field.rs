#![feature(staged_api)]
#![unstable(feature="field_root",issue="none")]
#[stable(feature="aggregate_api",since="1.0.0")]pub struct S{#[cfg(any())]#[stable(feature="field_api",since="1.0.0")]#[unstable(feature="field_next",issue="none")]pub removed:u8,#[stable(feature="field_api",since="1.0.0")]pub x:u8}fn main(){let _=S{x:7};}
