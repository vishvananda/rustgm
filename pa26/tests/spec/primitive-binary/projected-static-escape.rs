fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn escape<'a>(x:&'a <u8 as core::ops::Sub>::Output)->&'static u8{x}fn main(){}
