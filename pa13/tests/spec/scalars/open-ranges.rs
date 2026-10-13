fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
fn f(x:i8)->u8{match x{..=-1=>1,0.. =>2}}fn main(){check(f(-128)==1&&f(127)==2);}