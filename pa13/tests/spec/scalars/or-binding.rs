fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
fn f(x:u8)->u8{match x{n@1|n@3=>n+1,_=>0}}fn main(){check(f(1)==2&&f(3)==4&&f(2)==0);}