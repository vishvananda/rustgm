fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct K{key:i32,tag:u8}
impl PartialEq for K{fn eq(&self,o:&Self)->bool{self.key==o.key}}
impl Eq for K{}
impl PartialOrd for K{fn partial_cmp(&self,o:&Self)->Option<core::cmp::Ordering>{Some(self.cmp(o))}}
impl Ord for K{fn cmp(&self,o:&Self)->core::cmp::Ordering{self.key.cmp(&o.key)}}
static mut SUM:u32=0;impl Drop for K{fn drop(&mut self){unsafe{SUM=SUM+self.tag as u32;}}}fn main(){{let result=K{key:2,tag:1}.max(K{key:2,tag:2});check(result.tag==2);check(unsafe{SUM}==1);}check(unsafe{SUM}==3);}