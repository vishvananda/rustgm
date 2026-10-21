fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct Cell{n:u32}struct W(Cell);impl core::ops::Index<bool> for W{type Output=Cell;fn index(&self,_i:bool)->&Cell{&self.0}}fn main(){let mut w=W(Cell{n:1});w[false].n=9;}
