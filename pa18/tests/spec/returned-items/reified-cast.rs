fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn identity<T>(x:T)->T{x}fn id(x:u8)->u8{x+1}fn main(){let f=identity(id) as fn(u8)->u8;check(f(3)==4);}
