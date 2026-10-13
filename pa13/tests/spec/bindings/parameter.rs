fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
fn f((a,mut b):(u8,u64))->u64{b+=1;a as u64+b}fn main(){check(f((3,5))==9);}