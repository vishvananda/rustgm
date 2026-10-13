fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
enum V{}fn main(){let x:V;match x{_=>()}check(true);}