fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
const A:&[u8]=b"abc" as &[u8];const N:usize=match A{[_,_,_]=>3,_=>0};fn main(){check(N==3);check(A[1]==98);}
