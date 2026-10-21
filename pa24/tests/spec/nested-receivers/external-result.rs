fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn shared<F:Fn()->u8>(f:F)->u8{f()}fn mutable<F:FnMut()->u8>(mut f:F)->u8{f()}fn once<F:FnOnce()->u8>(f:F)->u8{f()}
fn result<F:FnOnce()->R,R>(f:F)->R{f()}fn take<'a,T>(x:&'a T)->&'a T{let f=move||x;let r=&f;result(&r)}fn main(){let n=7u8;check(*take(&n)==7);}
