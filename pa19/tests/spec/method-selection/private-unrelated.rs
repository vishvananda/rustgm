fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
mod m{pub struct S<T>(pub T);impl S<u8>{pub fn f()->u8{7}}impl S<u16>{fn f()->u8{9}}}fn main(){check(m::S::<u8>::f()==7);}