fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn make()->impl Fn()->u8{let n=7u8;||n}fn main(){let _=make();}
