fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn get<'a,T>(x:&'a T)->&'a T{let f=||x;f()}fn bad()->&'static u8{let a=7u8;get(&a)}fn main(){let _=bad();}
