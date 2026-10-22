fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
const A:&[u8]=b"";const B:&[u8]=A;const N:usize=B.len();fn main(){check(N==0);check(match b"" as &[u8]{B=>true,_=>false});check(match b"x" as &[u8]{B=>false,_=>true});}
