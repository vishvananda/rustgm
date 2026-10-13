fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
fn f(x:u8)->u8{match x{0..=9=>1,5..=20=>2,_=>3}}fn main(){check(f(7)==1&&f(15)==2&&f(30)==3);}