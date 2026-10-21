fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn call<F:FnOnce()->u8>(f:F)->u8{f()}fn main(){let mut n=5u8;let mut f=||{n+=1;n};check(call(&mut f)==6);check(f()==7);}
