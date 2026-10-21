fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn eq<T:Eq>(a:T,b:T)->bool{a==b}fn main(){let _=eq(1.0f64,1.0f64);}
