fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn bad<'a>()->&'a dyn Fn()->u8 { let f=||7; &f }fn main(){}
