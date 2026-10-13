fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
fn f(x:u8)->u8{let n@1..=9=x else{return 99;};n+1}fn main(){check(f(3)==4&&f(10)==99);}