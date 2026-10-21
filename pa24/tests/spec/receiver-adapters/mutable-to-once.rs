fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn call<F:FnOnce()->u8>(f:F)->u8{f()}fn main(){let mut n=6u8;let f=||{n+=1;n};check(call(f)==7);check(n==7);}
