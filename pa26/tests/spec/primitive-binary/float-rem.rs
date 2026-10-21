fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){check(<f64 as core::ops::Rem>::rem(7.5,2.0)==1.5);}
