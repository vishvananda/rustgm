fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
use std::cmp::PartialEq as Compare;struct S(u8);impl Compare for S{fn eq(&self,r:&Self)->bool{self.0==r.0}}fn main(){check(S(4)==S(4));check(core::cmp::PartialEq::ne(&S(4),&S(5)));}
