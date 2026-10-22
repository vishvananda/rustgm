fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
const A:&[u8]=b"abc";const B:&[u8]=A;const C:(&[u8],bool)=(B,true);const N:usize=B.len();fn pair(x:&[u8])->bool{match (x,true){C=>true,_=>false}}fn differs(x:&[u8])->bool{match x{B=>false,_=>true}}fn main(){check(N==3);check(pair(b"abc"));check(differs(b"abd"));}
