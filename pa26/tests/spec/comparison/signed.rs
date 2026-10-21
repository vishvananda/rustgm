fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn eq<T:PartialEq>(a:T,b:T)->bool{a==b}fn main(){check(eq(-127i8,-127i8));check(!eq(-1i64,1i64));}
