fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
static N:u64=42;static P:&u64=&N;const C:u64=*P;fn main(){check(C==42 && *P==42);}
