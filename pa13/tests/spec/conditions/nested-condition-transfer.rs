fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
fn f()->u8{let mut x=3u8;'outer:loop{while let n@1..=3=x{if n==2{break 'outer 7;}x-=1;}break 99;}}fn main(){check(f()==7);}