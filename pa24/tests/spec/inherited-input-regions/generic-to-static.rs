fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn f<'b,'c,T>(x:&'c &'b T)->&'static T{(|&s|s)(x)}fn main(){}
