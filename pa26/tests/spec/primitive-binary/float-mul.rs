fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){check(<f64 as core::ops::Mul>::mul(2.5,4.0)==10.0);}
