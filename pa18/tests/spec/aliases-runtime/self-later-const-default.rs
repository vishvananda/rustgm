fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
type A<T=A<u8>,const N:usize=3>=(T,[u8;N]);fn f(x:A)->u8{x.0.0+x.0.1[2]+x.1[1]}fn main(){check(f(((7,[1,2,3]),[4,5,6]))==15);}
