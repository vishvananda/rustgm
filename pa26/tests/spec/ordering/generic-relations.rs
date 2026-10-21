fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S(i32);impl PartialEq for S{fn eq(&self,r:&Self)->bool{self.0==r.0}}impl PartialOrd for S{fn partial_cmp(&self,r:&Self)->Option<core::cmp::Ordering>{if self.0<r.0{Some(core::cmp::Ordering::Less)}else if self.0>r.0{Some(core::cmp::Ordering::Greater)}else{Some(core::cmp::Ordering::Equal)}}}
fn less<T:PartialOrd>(a:T,b:T)->bool{a<b}fn main(){check(less(S(2),S(3)));check(less(-5i32,2));}
