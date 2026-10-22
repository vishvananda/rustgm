fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
const A:&[u8]=&[];const N:usize=A.len();fn main(){check(N==0);}
