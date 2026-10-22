fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
unsafe extern "C"{fn callback(f:unsafe extern "C" fn(u32,...)->i64)->i64;}
fn main(){check(unsafe{callback(va_provider::sum)}==15);}
