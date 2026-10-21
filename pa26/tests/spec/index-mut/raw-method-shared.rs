fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct Cell{n:u32}impl Cell{fn bump(&mut self){self.n+=1;}}struct W(*mut Cell);impl core::ops::Index<bool> for W{type Output=*mut Cell;fn index(&self,_i:bool)->&Self::Output{&self.0}}fn main(){let mut c=Cell{n:1};let w=W(&raw mut c);unsafe{(*w[false]).bump();}check(c.n==2);}
