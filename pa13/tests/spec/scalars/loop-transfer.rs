fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
fn f()->u8{let mut n=2;loop{match n{0=>break 7,_=>{n-=1;continue;}}}}fn main(){check(f()==7);}