#![allow(dead_code,unused_mut,invalid_reference_casting)]
const X:u32={let p={let a=[3u32,5];let r:&[u32]=&a;r as *const [u32]};let q=unsafe{&raw const (*p)[0]};let _=q;37};fn main(){let _=X;}
