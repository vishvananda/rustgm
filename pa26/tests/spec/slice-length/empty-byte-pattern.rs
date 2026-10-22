fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
const A:&[u8]=b"";const B:&[u8]=A;const N:usize=B.len();fn matches(x:&[u8])->bool{match x{B=>true,_=>false}}fn main(){check(N==0);check(matches(b""));check(!matches(b"x"));}
