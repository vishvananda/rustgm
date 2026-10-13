fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
fn main(){let ((n@(0|1),true|false)|(n@2..,true|false))=(7u8,false);check(n==7);}