fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
const fn tail(x:&[u8])->&[u8]{match x{[_,r @ ..]=>r,_=>x}}const A:&[u8]=tail(&[1,2,3]);const N:usize=A.len();fn main(){check(N==2);check(tail(&[1,2,3,4]).len()==3);}
