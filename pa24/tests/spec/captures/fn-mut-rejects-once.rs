fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S;fn invoke<F:FnMut()->S>(mut f:F)->S{f()}fn main(){let x=S;let _=invoke(||x);}
