fn check(ok:bool){if !ok{let _=failure(0);}}
fn failure(x:i32)->i32{1/x}
fn make()->(u8,u64){(3,99)}fn main(){check(make().1==99 && make().0==3);}
