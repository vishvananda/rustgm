fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
fn f(x:bool)->u8{let n=match x{true=>return 7,false=>9};n}fn main(){check(f(true)==7&&f(false)==9);}