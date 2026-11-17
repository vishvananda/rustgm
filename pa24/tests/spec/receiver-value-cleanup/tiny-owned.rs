fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct D(u8);impl Drop for D{fn drop(&mut self){}}fn call<F:FnMut()->D>(mut f:F)->D{f()}fn main(){let f=||D(7);let result=call(f);check(result.0==7);}
