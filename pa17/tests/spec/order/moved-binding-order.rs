fn trace(_:u8){}struct D(u8);impl Drop for D{fn drop(&mut self){trace(self.0);}}fn main(){let a;let b=D(1);a=b;let c=D(2);trace(9);}
