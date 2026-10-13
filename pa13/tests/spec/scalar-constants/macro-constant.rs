fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
const N:u8=3;macro_rules! choose{($x:expr)=>{match $x{N=>7,_=>9}}}fn main(){check(choose!(3u8)==7&&choose!(4u8)==9);}