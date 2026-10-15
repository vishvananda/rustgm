fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S<T>(T);impl<T>S<T>{fn f(self)->T{self.0}}fn main(){let s=S(7);let x:u8=s.f();check(x==7);}