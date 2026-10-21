fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
const F:fn(u8)->u8=|x|x+1;const X:u8=F(7);fn main(){}