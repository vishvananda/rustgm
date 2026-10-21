fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
const fn make()->fn(u8)->u8{|x|x+1}const F:fn(u8)->u8=make();fn main(){check(F(7)==8);}