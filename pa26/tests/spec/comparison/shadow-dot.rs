fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait PartialEq{fn eq(&self,r:&Self)->bool;}struct S;impl PartialEq for S{fn eq(&self,_:&Self)->bool{false}}impl core::cmp::PartialEq for S{fn eq(&self,_:&Self)->bool{true}}fn main(){check(!S.eq(&S));check(S==S);}
