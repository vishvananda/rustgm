fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct W(u32);impl core::ops::IndexMut<bool> for W{fn index_mut(&mut self,_i:bool)->&mut Self::Output{&mut self.0}}fn main(){}
