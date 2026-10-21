fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct D;impl Drop for D{fn drop(&mut self){}}fn consume(_:D){}fn call<F:FnOnce()->u8>(f:F)->u8{f()}fn main(){let d=D;let mut f=move||{consume(d);7u8};check(call(&mut f)==7);}
