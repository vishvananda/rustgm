fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn id<T>(x:T)->T{x}fn f<T>(x:T)->T{id(x)}fn main(){check(f(11u32)==11);}
