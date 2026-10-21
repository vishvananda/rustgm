fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S<'a>{a:&'static u8,b:&'a u8}fn make<'a,'b>(x:&'a S<'b>)->impl Fn()->&'static u8+'static{||x.a}fn main(){}