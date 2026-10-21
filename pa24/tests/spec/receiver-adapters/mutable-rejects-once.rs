fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct D(u8);fn call<F:FnMut()->D>(mut f:F)->D{f()}fn main(){let d=D(7);let f=||d;check(call(f).0==7);}
