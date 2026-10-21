fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn less<T:PartialOrd>(a:T,b:T)->bool{a<b}fn main(){check(less(-9i64,3));check(!less(255u8,0));check(less(false,true));check(less('a','z'));}
