fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct A(u8);struct B(u8);impl PartialEq<B> for A{fn eq(&self,r:&B)->bool{self.0==r.0}}impl PartialOrd<B> for A{fn partial_cmp(&self,r:&B)->Option<core::cmp::Ordering>{if self.0<r.0{Some(core::cmp::Ordering::Less)}else{Some(core::cmp::Ordering::Greater)}}}fn main(){let a=A(1);let b=B(2);check(a<b);check(a.0==1&&b.0==2);}
