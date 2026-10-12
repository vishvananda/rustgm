fn check(ok:bool){if !ok{let _=failure(0);}}
fn failure(x:i32)->i32{1/x}
fn main(){let x:u64=include!("value.rs");check(x==42);}
