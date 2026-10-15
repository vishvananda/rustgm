fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S(u8);impl S{fn f(&mut self,x:u8)->u8{self.0+=x;self.0}}fn main(){check(S(7).f(2)==9);}