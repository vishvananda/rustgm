fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
const fn cast<const N:usize>(p:&[u8;N])->&[u8]{p as &[u8]}const A:&[u8]=cast(&[3,5]);fn main(){check(A[1]==5);check(match A{[_,_]=>true,_=>false});}
