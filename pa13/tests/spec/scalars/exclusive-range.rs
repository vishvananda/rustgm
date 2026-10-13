fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
fn f(x:u8)->u8{match x{0..5=>1,5.. =>2}}fn main(){check(f(4)==1&&f(5)==2&&f(255)==2);}