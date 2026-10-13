fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
fn f(x:[u8;2])->u8{match x{[0,n]|[n,0]=>n,_=>99}}fn main(){check(f([0,7])==7&&f([8,0])==8&&f([8,9])==99);}