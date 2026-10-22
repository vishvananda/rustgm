#![allow(static_mut_refs)]
const fn count(a:&[u8])->u8{match a{[_,_,_,..]=>3,_=>0}}static mut A:[u8;3]=[7;3];const N:u8=unsafe{count(&A)};fn main(){let _=N;}
