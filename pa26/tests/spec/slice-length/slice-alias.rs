fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
type A=[u8];fn main(){check(A::len(&[1,2,3])==3);}
