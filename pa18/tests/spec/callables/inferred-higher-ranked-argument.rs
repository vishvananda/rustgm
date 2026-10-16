fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
type C=for<'a>fn(&'a u8);
fn pass<U>(f:fn(U))->fn(U){f}
fn take(f:C){let x=19;f(&x)}
fn cb(x:&u8){check(*x==19)}
fn main(){let f=pass::<_>(take);f(cb);}

