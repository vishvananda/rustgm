fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
const A:u8=B+1;fn f(x:u8)->u8{match x{A=>7,_=>9}}const B:u8=C;const C:u8=2;fn main(){check(f(3)==7&&f(2)==9);}