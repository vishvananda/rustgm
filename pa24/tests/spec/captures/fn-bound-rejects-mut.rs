fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn invoke<F:Fn()->u8>(f:F)->u8{f()}fn main(){let mut n=7u8;let _=invoke(||{n+=1;n});}
