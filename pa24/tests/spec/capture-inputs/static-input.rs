fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn make(x:&'static u8)->impl Fn()->&'static u8+'static{||&*x}static N:u8=7;fn main(){let f=make(&N);check(*f()==7);}