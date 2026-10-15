fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S<T>(T);impl<T>S<T>{fn get(self)->T{self.0}}fn main(){check(<S<_>>::get(S(17u16))==17);}