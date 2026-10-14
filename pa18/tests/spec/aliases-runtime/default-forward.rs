fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
type A<T,U=T>=(T,U);type B<V=u8>=A<V>;fn f(x:B)->u8{x.0+x.1}fn main(){check(f((7,8))==15);}
