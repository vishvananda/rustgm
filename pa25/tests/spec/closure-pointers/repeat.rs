fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
const F:[fn(u8)->u8;3]=[|x|x+2;3];fn main(){check(F[2](7)==9);}