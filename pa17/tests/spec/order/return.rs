fn trace(_:u8){}struct D(u8);impl Drop for D{fn drop(&mut self){trace(self.0);}}fn f()->D{let a=D(1);return D(2);}fn main(){let result=f();trace(9);}
