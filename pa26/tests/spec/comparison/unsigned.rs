fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn eq<T:Eq>(a:T,b:T)->bool{a==b}fn main(){check(eq(255u8,255u8));check(!eq(0u64,1u64));}
