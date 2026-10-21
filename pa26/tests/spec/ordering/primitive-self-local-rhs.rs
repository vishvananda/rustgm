fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S(u8);impl PartialEq<S> for u8{fn eq(&self,r:&S)->bool{*self==r.0}}impl PartialOrd<S> for u8{fn partial_cmp(&self,r:&S)->Option<core::cmp::Ordering>{if *self<r.0{Some(core::cmp::Ordering::Less)}else{None}}}fn main(){let a=1;check(a<S(2));}
