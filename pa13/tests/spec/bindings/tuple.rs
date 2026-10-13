fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
fn main(){let (x,y)=(10u64,7u8);check(x==10&&y==7);}