fn trace(_:u8){}struct D(u8);impl Drop for D{fn drop(&mut self){trace(self.0);}}fn make(n:u8)->D{trace(n+10);D(n)}fn main(){let mut a=make(1);a=make(2);trace(9);}
