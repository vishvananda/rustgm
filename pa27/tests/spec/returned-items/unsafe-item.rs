fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn identity<T>(x:T)->T{x}unsafe fn id(x:u8)->u8{x+1}fn main(){check(unsafe{identity(id)(3)}==4);}
