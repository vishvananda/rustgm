fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct D<'a>(&'a mut u8);fn call<F:FnOnce()->u8>(f:F)->u8{f()}fn main(){let mut n=7u8;let d=D(&mut n);let f=move||*d.0;check(call(f)==7);check(call(f)==7);}
