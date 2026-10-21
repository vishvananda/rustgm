struct S(u8); impl S { fn get(&self)->u8{self.0} fn inc(&mut self){self.0+=1;} }
fn main(){let mut s=S(7);let r=&mut s;let mut f=||r.inc();f();}
