fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
const F:unsafe extern "C" fn(u32,...)->i64=va_provider::sum;
fn main(){check(unsafe{F(2,17i64,19i64)}==36);}
