fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
fn f(x:u8)->u8{let n=17;let n@0..=9=x else{return n;};n}fn main(){check(f(3)==3&&f(20)==17);}