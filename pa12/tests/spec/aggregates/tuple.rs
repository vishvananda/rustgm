fn check(ok:bool){if !ok{let _=failure(0);}} fn failure(x:i32)->i32{1/x}
fn pair(a:u64,b:u8)->(u64,u8){(a,b)} fn main(){let p=pair(42,7);check(p.0==42 && p.1==7);}
