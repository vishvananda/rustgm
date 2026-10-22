#![allow(dead_code,unused_mut,invalid_reference_casting)]
const X:()={let a=[();2];let r:&[()]=&a;let p=r as *const [()];unsafe{(*p)[1]}};fn main(){let _=X;}
