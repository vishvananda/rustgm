fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct X(u8);impl PartialEq for X{fn eq(&self,_:&Self)->bool{true}fn ne(&self,_:&Self)->bool{true}}fn main(){let a=[X(1)];let b=[X(2)];check(!(a==b));check(a!=b);check(a[0]!=b[0]);}
