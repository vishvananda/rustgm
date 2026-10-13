fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
fn f()->u8{let (a,b):(u8,u8)=return 3;a+b}fn main(){check(f()==3);}