fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct Z;type A<T,const N:usize>=[T;N];fn take(_:A<Z,2>){}fn main(){take([Z,Z]);}
