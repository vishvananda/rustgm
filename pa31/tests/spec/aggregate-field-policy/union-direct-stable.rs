#![feature(staged_api)]
#![unstable(feature="field_root",issue="none")]
#[stable(feature="aggregate_api",since="1.0.0")]pub union S{#[stable(feature="field_api",since="1.0.0")]pub x:u8}fn main(){let x=S{x:7};let _=unsafe{x.x};}
