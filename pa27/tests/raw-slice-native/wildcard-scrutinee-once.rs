#![allow(dead_code,unused_mut,invalid_reference_casting)]
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
static mut COUNT:u32=0;static A:[u32;3]=[3,5,7];fn make()->*const [u32]{unsafe{COUNT+=1;}let r:&[u32]=&A;r as *const [u32]}fn main(){let x=unsafe{match *make(){[_,_,_]=>37,_=>0}};check(x==37);check(unsafe{COUNT}==1);}
