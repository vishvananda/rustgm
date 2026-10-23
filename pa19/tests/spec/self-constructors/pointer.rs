fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S<T>(T);impl<T>S<T>{fn factory()->fn(T)->Self{Self}}fn main(){check(S::<u16>::factory()(517).0==517);}
