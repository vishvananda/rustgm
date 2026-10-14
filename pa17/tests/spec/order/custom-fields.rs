fn trace(_:u8){}struct D(u8);impl Drop for D{fn drop(&mut self){trace(self.0);}}struct S(D,D);impl Drop for S{fn drop(&mut self){trace(8);}}fn main(){let s=S(D(1),D(2));trace(9);}
