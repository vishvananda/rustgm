fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct K{key:i32,tag:u8}
impl PartialEq for K{fn eq(&self,o:&Self)->bool{self.key==o.key}}
impl Eq for K{}
impl PartialOrd for K{fn partial_cmp(&self,o:&Self)->Option<core::cmp::Ordering>{Some(self.cmp(o))}}
impl Ord for K{fn cmp(&self,o:&Self)->core::cmp::Ordering{self.key.cmp(&o.key)}}
static mut SEQ:u32=0;fn make(key:i32,tag:u8)->K{unsafe{SEQ=SEQ*10+tag as u32;}K{key,tag}}fn main(){check(make(1,1).max(make(2,2)).tag==2);check(unsafe{SEQ}==12)}