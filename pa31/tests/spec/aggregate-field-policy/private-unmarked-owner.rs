#![feature(staged_api)]
#![stable(feature="field_root",since="1.0.0")]
struct S(pub u8);fn main(){let _=S(7).0;}
