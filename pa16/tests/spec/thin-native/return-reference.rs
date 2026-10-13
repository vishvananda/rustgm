fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn id<'a>(x:&'a u8)->&'a u8{x}fn main(){let x=42;check(*id(&x)==42);}
