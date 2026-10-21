fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
static F:[fn(u8)->u8;2]=[|x|x+1,|x|x+2];fn main(){check(F[0](7)==8);check(F[1](7)==9);}