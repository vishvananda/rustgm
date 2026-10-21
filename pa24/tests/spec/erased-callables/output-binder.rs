fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn run(f:&dyn for<'a> Fn()->&'a u8){}fn main(){}
