fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let f:unsafe extern "C" fn(u32,...)->i64=va_provider::sum;check(unsafe{f(2,9i64,5i64)}==14);}
