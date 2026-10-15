fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S<T>(T);impl S<*const u8>{fn f()->u8{7}}impl S<*mut u8>{fn f()->u8{9}}fn main(){check(S::<*const _>::f()==7);}