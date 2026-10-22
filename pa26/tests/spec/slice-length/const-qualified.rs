fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
const N:usize= <[u8]>::len(&[1,2,3]);fn main(){check(N==3);}
