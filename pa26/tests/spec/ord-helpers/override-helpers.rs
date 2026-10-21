fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct K{key:i32,tag:u8}
impl PartialEq for K{fn eq(&self,o:&Self)->bool{self.key==o.key}}
impl Eq for K{}
impl PartialOrd for K{fn partial_cmp(&self,o:&Self)->Option<core::cmp::Ordering>{Some(self.cmp(o))}}
impl Ord for K{fn cmp(&self,_:&Self)->core::cmp::Ordering{trap()}fn min(self,o:Self)->Self{o}fn max(self,_:Self)->Self{self}fn clamp(self,_:Self,o:Self)->Self{o}}
fn main(){check(K{key:1,tag:1}.min(K{key:2,tag:2}).tag==2);check(K{key:1,tag:1}.max(K{key:2,tag:2}).tag==1);check(K{key:1,tag:1}.clamp(K{key:2,tag:2},K{key:3,tag:3}).tag==3)}
fn trap<T>()->T{let _=fail(0);loop{}}
