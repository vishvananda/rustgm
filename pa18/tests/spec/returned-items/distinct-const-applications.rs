fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn identity<T>(x:T)->T{x}fn add<const N:u8>(x:u8)->u8{x+N}fn main(){check(identity(add::<3>)(4)==7);check(identity(add::<5>)(4)==9);}
