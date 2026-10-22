fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
const N:usize=[1u8,2,3].len();fn main(){check(N==3);}
