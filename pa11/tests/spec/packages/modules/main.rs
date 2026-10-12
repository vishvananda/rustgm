fn check(ok:bool){if !ok{let _=failure(0);}}
fn failure(x:i32)->i32{1/x}
mod child;fn main(){check(child::value(41)==42);}
