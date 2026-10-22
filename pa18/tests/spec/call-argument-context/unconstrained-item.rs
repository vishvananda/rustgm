fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn keep<T>(_:T){}fn id(x:u8)->u8{x}fn main(){keep(id);check(true);}
