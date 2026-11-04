#![feature(staged_api)]
#![unstable(feature="field_root",issue="none")]
#[stable(feature="aggregate_api",since="1.0.0")]pub struct S(#[unstable(feature="field_next",issue="none")]pub u8);#[stable(feature="aggregate_api",since="1.0.0")]#[rustc_const_stable(feature="read_api",since="1.0.0")]pub const fn read(s:&S)->u8{s.0}const N:u8=read(&S(7));fn main(){let _=N;}
