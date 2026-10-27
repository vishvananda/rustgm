fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct W([u32;2]);impl core::ops::Index<usize> for W{type Output=u32;fn index(&self,i:usize)->&u32{&self.0[i]}}impl core::ops::IndexMut<usize> for W{fn index_mut(&mut self,i:usize)->&mut u32{&mut self.0[i]}}
fn main(){let mut w=W([3,5]);{let mut f=|i:usize|{w[i]+=1;};f(1);f(0);}check(w.0[0]==4);check(w.0[1]==6);}
