fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
extern crate first;const R:&u32=first::REF;const N:u32=*R;fn main(){check(N==7);check(R as *const u32==&first::IMM as *const u32);}