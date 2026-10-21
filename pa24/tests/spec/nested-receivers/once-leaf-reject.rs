fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn shared<F:Fn()->u8>(f:F)->u8{f()}fn mutable<F:FnMut()->u8>(mut f:F)->u8{f()}fn once<F:FnOnce()->u8>(f:F)->u8{f()}
struct D(u8);fn main(){let d=D(7);let f=move||{let old=d;old.0};let r=&f;let _=once(&r);}
