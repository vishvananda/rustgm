fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
fn f(x:(f32,u8))->u8{match x{(_,n)=>n}}fn main(){check(f((2.0,7))==7);}