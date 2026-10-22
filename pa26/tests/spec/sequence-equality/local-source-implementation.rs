fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct X(u8);impl PartialEq<[u8;1]> for X{fn eq(&self,b:&[u8;1])->bool{self.0==b[0]}}fn main(){check(X(3)==[3]);}
