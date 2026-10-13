fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
fn f()->u8{let mut x=3u8;let mut sum=0;while let n@1..=3=x{sum+=n;x-=1;}sum}fn main(){check(f()==6);}