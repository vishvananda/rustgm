fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S(usize);type A<const N:usize={S(3).0}>=[u8;N];fn read(x:A)->u8{x[2]}fn main(){check(read([2,5,9])==9);}
