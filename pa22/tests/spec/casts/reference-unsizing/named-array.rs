fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
const B:[u8;3]=[3,5,7];const A:&[u8]=&B as &[u8];fn main(){check(A[2]==7);check(match A{[_,_,_]=>true,_=>false});}
