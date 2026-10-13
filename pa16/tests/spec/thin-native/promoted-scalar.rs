fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn f()->&'static u64{&(40+2)}fn main(){check(*f()==42);}
