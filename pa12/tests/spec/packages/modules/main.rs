fn check(ok:bool){if !ok{let _=failure(0);}}
fn failure(x:i32)->i32{1/x}
mod items;fn main(){let s=items::make(42);check(s.value==42);}
