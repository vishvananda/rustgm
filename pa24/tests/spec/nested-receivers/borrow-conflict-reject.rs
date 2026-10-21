fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn shared<F:Fn()->u8>(f:F)->u8{f()}fn mutable<F:FnMut()->u8>(mut f:F)->u8{f()}fn once<F:FnOnce()->u8>(f:F)->u8{f()}
fn main(){let mut n=7u8;let mut f=||{n+=1;n};let mut r=&mut f;let s=&mut r;let other=&mut f;let _=once(s);let _=other();}
