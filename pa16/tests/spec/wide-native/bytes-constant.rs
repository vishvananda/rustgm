fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
const S:&[u8]=b"abc";fn main(){check(S[1]==98);}
