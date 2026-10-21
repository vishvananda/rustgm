fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn once<F:FnOnce()->R,R>(f:F)->R{f()}fn twice<F:FnMut()->R,R>(mut f:F)->R{let _=f();f()}fn take<T:Copy>(x:T){let mut a=x;let f=||{a=x;};let p=&a;once(f);let _=*p;}fn main(){take(3u8);}
