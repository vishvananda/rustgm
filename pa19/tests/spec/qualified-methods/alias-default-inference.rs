fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S<T>(T);type A<T=u8>=S<T>;impl<T>S<T>{fn get(self)->T{self.0}}fn main(){let x:u16=A::get(S(517));check(x==517);}