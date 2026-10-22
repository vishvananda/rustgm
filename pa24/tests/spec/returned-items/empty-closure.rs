fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn identity<T>(x:T)->T{x}fn main(){check(identity(||3)()==3);}
