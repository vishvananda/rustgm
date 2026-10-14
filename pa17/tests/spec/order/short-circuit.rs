fn trace(_:u8){}struct D(u8);impl Drop for D{fn drop(&mut self){trace(self.0);}}fn main(){if {D(1);false}&&{D(2);true}{trace(8);}trace(9);}
