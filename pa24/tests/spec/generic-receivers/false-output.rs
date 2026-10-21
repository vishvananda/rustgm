fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn once<F:FnOnce()->R,R>(f:F)->R{f()}fn twice<F:FnMut()->R,R>(mut f:F)->R{let _=f();f()}fn bad<F:FnOnce()->u16>(f:F)->u16{f()}fn take<T:Copy>(x:T){let f=move||{let _=x;7u8};let _=bad(f);}fn main(){take(3u8);}
