fn check(v:bool){if !v{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn drive<R,F:FnOnce()->R>(f:F)->R{f()}
fn bad<T:'static>(x:T)->&'static T{let f=||(&x,);drive(f).0}
fn main(){let _=bad(23u8);}
