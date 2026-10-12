fn check(ok:bool){if !ok{let _=failure(0);}} fn failure(x:i32)->i32{1/x}
fn main(){let mut a=[1u8,2];a[0]+={a[0]=9;3};check(a[0]==12);}
