fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
fn f(x:u8)->u8{if let mut n@0..=9=x{n+=1;n}else{99}}fn main(){check(f(7)==8&&f(20)==99);}