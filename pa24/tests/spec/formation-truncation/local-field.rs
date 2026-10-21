fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S<'b>{a:&'b u8}fn make<'a,'b>(x:&'a S<'b>)->impl Fn()->u8+'static{||*x.a}fn main(){}