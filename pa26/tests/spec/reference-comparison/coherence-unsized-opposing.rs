fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S<T:?Sized>(T);impl<T> PartialEq for S<T>{fn eq(&self,_:&Self)->bool{true}}impl PartialEq for &S<[u8]>{fn eq(&self,_:&Self)->bool{true}}fn main(){}
