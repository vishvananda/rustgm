fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
const S:&[u8]=&[7,9];fn main(){check(S[0]==7 && S[1]==9);}
