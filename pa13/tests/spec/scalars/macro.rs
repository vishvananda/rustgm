fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
macro_rules! choose{($x:expr,$p:pat)=>{match $x{$p=>7,_=>9}}}fn main(){check(choose!(1u8,0|1)==7&&choose!(2u8,0|1)==9);}