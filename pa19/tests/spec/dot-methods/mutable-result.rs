fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S(u8);impl S{fn f(&mut self)->&mut u8{&mut self.0}}fn main(){let mut s=S(7);*s.f()=9;check(s.0==9);}