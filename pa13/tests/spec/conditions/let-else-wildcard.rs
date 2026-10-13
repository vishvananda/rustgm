fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
fn main(){let x:u8;let _=x else{return;};let 0..=255=x else{return;};check(true);}