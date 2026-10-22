fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
extern crate first;fn main(){let a=&first::IMM;let b=first::REF;check(a as *const u32==b as *const u32);check(*b==7);}