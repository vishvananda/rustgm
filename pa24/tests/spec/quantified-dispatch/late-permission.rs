fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn bad<'a>(x:&'a u8){let f=move|_:&u8|x;let p:&dyn for<'b>Fn(&'b u8)->&'a u8=&f;let _=p(&0);let q:&dyn for<'b>Fn(&'b u8)->&'static u8=&f;let _=q;}fn main(){}
