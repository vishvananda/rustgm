fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S(u8);fn main(){let s=S(7);let f=||s.0+2;check(f()==9);check(s.0==7);}
