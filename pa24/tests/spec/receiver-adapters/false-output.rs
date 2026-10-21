fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn call<F:FnOnce()->u16>(f:F)->u16{f()}fn main(){let n=7u8;let f=||n;check(call(f)==7);}
