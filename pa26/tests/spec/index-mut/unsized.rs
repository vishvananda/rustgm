fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct W([u8;2]);impl core::ops::Index<bool> for W{type Output=[u8];fn index(&self,_i:bool)->&[u8]{&self.0}}impl core::ops::IndexMut<bool> for W{fn index_mut(&mut self,_i:bool)->&mut[u8]{&mut self.0}}fn main(){let mut w=W([1,2]);w[false][1]=9;check(w.0[1]==9);}
