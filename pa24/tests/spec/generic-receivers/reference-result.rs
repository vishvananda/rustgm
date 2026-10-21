fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn once<F:FnOnce()->R,R>(f:F)->R{f()}fn twice<F:FnMut()->R,R>(mut f:F)->R{let _=f();f()}fn take<'a,T>(x:&'a T)->&'a T{let f=move||x;once(&f)}fn main(){let x=7u8;check(*take(&x)==7);}
