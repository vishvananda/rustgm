fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S(u8);impl PartialEq for S{fn eq(&self,o:&Self)->bool{self.0==o.0}fn ne(&self,_:&Self)->bool{false}}fn main(){let a=S(7);let b=S(7);let a=&a;let b=&b;check(a.eq(&b));check(!a.ne(&b));}
