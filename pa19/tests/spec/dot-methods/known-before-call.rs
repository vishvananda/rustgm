fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S<T>(T);impl S<u8>{fn f(self)->u8{7}}impl S<u16>{fn f(self)->u8{9}}fn main(){check(S(1u8).f()==7);}