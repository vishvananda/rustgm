fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
fn f(x:u8)->u8{match x{a@b@(0|1)=>a+b,_=>0}}fn main(){check(f(1)==2);}