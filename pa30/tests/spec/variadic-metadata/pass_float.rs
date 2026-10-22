fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){check(unsafe{va_provider::sum_float(9,1.0f64,2.0f64,3.0f64,4.0f64,5.0f64,6.0f64,7.0f64,8.0f64,9.0f64)}==45.0);}
