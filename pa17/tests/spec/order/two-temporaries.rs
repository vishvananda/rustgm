fn trace(_:u8){}struct D(u8);impl Drop for D{fn drop(&mut self){trace(self.0);}}fn main(){let a=(&D(1),D(2));let b=(&D(3),D(4));trace(9);}
