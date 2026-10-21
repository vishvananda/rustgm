fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S(u8);impl PartialEq for S{fn eq(&self,r:&Self)->bool{self.0==r.0}}fn main(){check(<S as PartialEq>::eq(&S(3),&S(3)));check(<S as PartialEq>::ne(&S(3),&S(4)));check(<u8 as PartialEq>::eq(&3,&3));check(<u8 as PartialEq>::ne(&3,&4));}
