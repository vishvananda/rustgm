fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S;impl PartialEq<str> for S{fn eq(&self,_:&str)->bool{false}}impl PartialOrd<str> for S{fn partial_cmp(&self,_:&str)->Option<core::cmp::Ordering>{Some(core::cmp::Ordering::Less)}}fn main(){check(S.lt("x"));}
