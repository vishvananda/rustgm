fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
extern crate first;const X:u32=first::IMM;const Y:u32=first::RECORD.n;fn main(){check(X==7);check(Y==17);}