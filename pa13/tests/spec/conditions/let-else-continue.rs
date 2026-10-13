fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
fn f()->u8{let mut n=4;loop{n-=1;let 0=n else{continue;};break 7;}}fn main(){check(f()==7);}