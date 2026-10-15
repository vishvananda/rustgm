fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S<T>(T);impl S<fn(u8)->u16>{fn f()->u8{7}}impl S<fn(u16)->u8>{fn f()->u8{9}}fn main(){check(S::<fn(u8)->_>::f()==7);}