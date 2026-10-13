fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
static A:[u8;2]=[7,9];static S:&[u8]=&A;fn main(){check(S[0]==7 && S[1]==9);}
