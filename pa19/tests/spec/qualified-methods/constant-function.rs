fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S(u8);impl S{fn get(self)->u8{self.0}}const F:fn(S)->u8=<S>::get;fn main(){check(F(S(23))==23);}