fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn take<T:Copy>(x:T)->&'static T{let f=||&x;f()}fn main(){let _=take(7u8);}
