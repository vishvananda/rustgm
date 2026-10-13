fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
fn f()->u8{let mut x=4u8;while let n@1..=4=x{if n==2{break;}x-=1;}x}fn main(){check(f()==2);}