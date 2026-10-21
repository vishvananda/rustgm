fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
const C:u8=7;fn main(){check(C==7);}
