fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S(u8);impl S{fn f(self:&&Self)->u8{self.0}}fn main(){let s=S(7);check((&s).f()==7);}