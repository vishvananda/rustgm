fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
const A:&[u8]=b"abc";const B:&[u8]=A;const C:(&[u8],bool)=(B,true);const N:usize=B.len();fn main(){check(N==3);check(match (b"abc" as &[u8],true){C=>true,_=>false});check(match b"abd" as &[u8]{B=>false,_=>true});}
