fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S(u8);impl PartialEq for S{fn eq(&self,_:&Self)->bool{false}fn ne(&self,_:&Self)->bool{false}}fn main(){check(!(S(1)==S(1)));check(!(S(1)!=S(2)));}
