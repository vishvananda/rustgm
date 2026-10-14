fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S<T>(T);fn factory<T>()->fn(T)->S<T>{S::<T>}fn main(){let a=factory::<u16>()(503);check(a.0==503);}
