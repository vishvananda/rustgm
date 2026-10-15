fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
#[derive(Copy,Clone)]struct S(u8);impl S{fn get(self)->u8{self.0}}fn main(){let s=S(7);let r=&&s;check(r.get()==7);check(s.0==7);}