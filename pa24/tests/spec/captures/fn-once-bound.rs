fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S(u8);fn invoke<F:FnOnce()->S>(f:F)->S{f()}fn main(){let x=S(7);check(invoke(||x).0==7);}
