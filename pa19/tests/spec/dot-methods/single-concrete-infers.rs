fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S<T>(T);impl S<u8>{fn f(self)->u8{self.0}}fn main(){check(S(7).f()==7);}