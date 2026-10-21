fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn once<F:FnOnce()->R,R>(f:F)->R{f()}fn twice<F:FnMut()->R,R>(mut f:F)->R{let _=f();f()}fn run<F:for<'a>FnOnce()->u8>(f:F)->u8{f()}fn take<T:Copy>(x:T)->u8{let f=move||{let _v=x;7};run(f)}fn main(){check(take(3u8)==7);}
