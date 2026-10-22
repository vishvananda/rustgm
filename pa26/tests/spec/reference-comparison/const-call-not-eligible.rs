fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
const X:bool=<&u8 as PartialEq>::eq(&&1u8,&&1u8);fn main(){check(X);}
