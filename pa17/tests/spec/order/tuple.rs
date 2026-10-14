fn trace(_:u8){}struct D(u8);impl Drop for D{fn drop(&mut self){trace(self.0);}}fn main(){let p=(D(1),D(2),D(3));trace(9);}
