fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S<T,U>(T,U);type A<T>=S<T,T>;impl S<u8,u16>{fn f()->u8{9}}impl S<u8,u8>{fn f()->u8{7}}fn main(){check(A::<_>::f()==7);}