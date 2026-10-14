fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn id<T>(x:T)->T{x}fn main(){let f=id;check(f(15u8)==15);}
