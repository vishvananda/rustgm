fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
fn f(x:u8)->u8{match x{0|_=>7,_=>99}}fn main(){check(f(0)==7&&f(3)==7);}