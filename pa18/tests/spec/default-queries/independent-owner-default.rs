type A<T=A<u8>>=T;type B<T=A<u16>>=T;fn f(x:A,y:B)->(u8,u16){(x,y)}
fn main(){}
