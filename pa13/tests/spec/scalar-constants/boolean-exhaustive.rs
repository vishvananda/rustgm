fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
const YES:bool=true;const NO:bool=!YES;fn f(x:bool)->u8{match x{YES=>7,NO=>9}}fn main(){check(f(true)==7&&f(false)==9);}