unsafe extern "C"{fn sum_float(n:u32,...)->f64;}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){check(unsafe{sum_float(10,1.0f64,2.0f64,3.0f64,4.0f64,5.0f64,6.0f64,7.0f64,8.0f64,9.0f64,10.0f64)}==55.0);}
