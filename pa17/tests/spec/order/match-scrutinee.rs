fn trace(_:u8){}struct D(u8);impl Drop for D{fn drop(&mut self){trace(self.0);}}fn main(){match D(1){ref d=>trace(8)}trace(9);}
