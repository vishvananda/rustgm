fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S;impl PartialEq for S{fn eq(&self,_:&Self)->bool{false}}impl Eq for S{}fn main(){check(!(S==S));}
