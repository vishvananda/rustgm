fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S<T>(T);type A<T>=S<T>;impl<T>S<T>{fn get(self)->T{self.0}}fn main(){check(<A<u8>>::get(S(19))==19);}