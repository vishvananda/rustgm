fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S(u8);impl S{fn set(&mut self,x:u8){self.0=x;}}fn main(){let mut s=S(1);S::set(&mut s,9);check(s.0==9);}