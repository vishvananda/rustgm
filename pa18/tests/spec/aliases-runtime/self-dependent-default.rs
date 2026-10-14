fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
type A<T=A<u16>,U=T>=(T,U);fn sum(x:A)->u16{x.0.0+x.0.1+x.1.0+x.1.1}fn main(){check(sum(((1,2),(3,4)))==10);}
