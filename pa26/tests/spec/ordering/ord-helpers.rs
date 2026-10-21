fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S(i32);impl PartialEq for S{fn eq(&self,r:&Self)->bool{self.0==r.0}}impl PartialOrd for S{fn partial_cmp(&self,r:&Self)->Option<core::cmp::Ordering>{if self.0<r.0{Some(core::cmp::Ordering::Less)}else if self.0>r.0{Some(core::cmp::Ordering::Greater)}else{Some(core::cmp::Ordering::Equal)}}}
impl Eq for S{}impl Ord for S{fn cmp(&self,r:&Self)->core::cmp::Ordering{if self.0<r.0{core::cmp::Ordering::Less}else if self.0>r.0{core::cmp::Ordering::Greater}else{core::cmp::Ordering::Equal}}}
fn main(){check(S(1).max(S(2)).0==2);check(S(2).min(S(1)).0==1);check(S(9).clamp(S(1),S(3)).0==3);}
