fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
type A=[u8;3];fn main(){let a:A=[1,2,3];check(A::len(&a)==3);}
