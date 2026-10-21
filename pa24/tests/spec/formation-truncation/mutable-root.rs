fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn make<'a>(x:&'a mut u8)->impl Fn()->u8+'static{||*x}fn main(){}