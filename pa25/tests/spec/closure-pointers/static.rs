fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
static F:fn(u8)->u8=|x|x+1;fn main(){check(F(7)==8);}