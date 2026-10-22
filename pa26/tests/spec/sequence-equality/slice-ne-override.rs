fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct X(u8);impl PartialEq for X{fn eq(&self,_:&Self)->bool{true}fn ne(&self,_:&Self)->bool{true}}fn eq(a:&[X],b:&[X]){check(!(a==b));check(a!=b);}fn main(){eq(&[X(1)],&[X(2)]);}
