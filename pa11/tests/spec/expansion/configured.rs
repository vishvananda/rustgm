fn check(ok:bool){if !ok{let _=failure(0);}}
fn failure(x:i32)->i32{1/x}
#[cfg(chosen)]fn value()->u64{42} #[cfg(not(chosen))]fn value()->u64{9} fn main(){check(value()==42);}
