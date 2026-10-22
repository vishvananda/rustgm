#![allow(dead_code,unused_mut,invalid_reference_casting)]
const X:u32={let p={let a=[();2];let r:&[()]=&a;r as *const [()]};let q=unsafe{&raw const (*p)[1]};let _=q;37};fn main(){let _=X;}
