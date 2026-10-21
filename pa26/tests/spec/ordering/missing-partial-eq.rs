fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S;impl PartialOrd for S{fn partial_cmp(&self,_:&Self)->Option<core::cmp::Ordering>{None}}fn main(){}
