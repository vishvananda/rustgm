fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S;impl PartialEq for S{fn eq(&self,_:&Self)->bool{false}}impl PartialOrd for S{fn partial_cmp(&self,_:&Self)->Option<core::cmp::Ordering>{let _=fail(0);None}fn lt(&self,_:&Self)->bool{true}}fn main(){check(S<S);}
