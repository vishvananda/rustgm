fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
fn main(){let xs=[();18446744073709551615];let[a,tail@..,b]=xs;let _=(a,tail,b);check(true);}