fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn eq<T:PartialEq>(a:T,b:T)->bool{a==b}fn main(){let z=0.0f64;let n=z/z;check(!eq(n,n));check(<f64 as PartialEq>::ne(&n,&n));check(eq(-0.0f32,0.0f32));}
