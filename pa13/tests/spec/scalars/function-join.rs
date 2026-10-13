fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
fn a(x:u8)->u8{x+1}fn b(x:u8)->u8{x+2}fn f(x:bool)->u8{(match x{true=>a,false=>b})(3)}fn main(){check(f(true)==4&&f(false)==5);}