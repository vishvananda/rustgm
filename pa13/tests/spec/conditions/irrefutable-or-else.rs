fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
fn f(x:u8)->u8{let (n@0|n)=x else{return 99;};n}fn main(){check(f(0)==0&&f(7)==7);}