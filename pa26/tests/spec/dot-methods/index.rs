fn verify(ok:bool){if !ok{let _=fault(0);}}fn fault(x:u8)->u8{1/x}
use core::ops::Index;struct W(u8);impl core::ops::Index<u8> for W{type Output=u8;fn index(&self,_i:u8)->&u8{&self.0}}impl core::ops::IndexMut<u8> for W{fn index_mut(&mut self,_i:u8)->&mut u8{&mut self.0}}
fn main(){let w=W(7);verify(*w.index(0)==7);}
