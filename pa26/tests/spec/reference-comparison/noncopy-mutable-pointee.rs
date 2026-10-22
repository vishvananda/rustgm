fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct K{key:i32,tag:u8}
impl PartialEq for K{fn eq(&self,o:&Self)->bool{self.key==o.key}}
impl Eq for K{}
impl PartialOrd for K{fn partial_cmp(&self,o:&Self)->Option<core::cmp::Ordering>{Some(self.cmp(o))}}
impl Ord for K{fn cmp(&self,o:&Self)->core::cmp::Ordering{self.key.cmp(&o.key)}}
fn main(){let mut a=K{key:1,tag:1};let mut b=K{key:1,tag:2};let a=&mut a;let b=&mut b;check(<&mut K as PartialEq>::eq(&a,&b));a.tag=7;b.tag=8;check(a.tag==7&&b.tag==8);}
