fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
fn f(x:bool)->u8{match x{true=>3,false=>7}}fn main(){check(f(true)==3&&f(false)==7);}