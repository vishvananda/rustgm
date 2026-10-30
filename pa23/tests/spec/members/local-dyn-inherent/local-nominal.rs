fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct Number(u8);impl Number{fn value(&self)->u8{self.0}}fn main(){let n=Number(7);check(n.value()==7);}
