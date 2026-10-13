fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
fn f(c:bool)->u8{let x;if c{x=7}else{x=3}x}fn main(){check(f(true)==7&&f(false)==3);}
