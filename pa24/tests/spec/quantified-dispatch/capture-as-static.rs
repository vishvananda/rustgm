fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn bad<'a>(x:&'a u8){let p:&dyn for<'b>Fn(&'b u8)->&'static u8=&move|_|x;let _=p;}fn main(){}
