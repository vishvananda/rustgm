fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct K{key:i32,tag:u8}
impl PartialEq for K{fn eq(&self,o:&Self)->bool{self.key==o.key}}
impl Eq for K{}
impl PartialOrd for K{fn partial_cmp(&self,_:&Self)->Option<core::cmp::Ordering>{trap()}fn lt(&self,_:&Self)->bool{false}fn le(&self,_:&Self)->bool{false}fn gt(&self,_:&Self)->bool{false}fn ge(&self,_:&Self)->bool{false}}
impl Ord for K{fn cmp(&self,o:&Self)->core::cmp::Ordering{self.key.cmp(&o.key)}}
fn main(){check(K{key:1,tag:1}.min(K{key:2,tag:2}).tag==1);check(K{key:1,tag:1}.max(K{key:2,tag:2}).tag==2)}
fn trap<T>()->T{let _=fail(0);loop{}}
