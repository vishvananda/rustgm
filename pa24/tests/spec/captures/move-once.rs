fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S(u8);fn main(){let s=S(7);let f=||s;let t=f();check(t.0==7);}
