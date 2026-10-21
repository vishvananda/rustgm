fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){check(<i128 as core::ops::Add>::add(-18446744073709551616,7)==-18446744073709551609);}
