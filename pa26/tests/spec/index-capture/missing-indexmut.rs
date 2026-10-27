fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct W([u32;2]);impl core::ops::Index<usize> for W{type Output=u32;fn index(&self,i:usize)->&u32{&self.0[i]}}fn main(){let mut w=W([3,5]);let mut f=||{w[0]+=1;};f();}
