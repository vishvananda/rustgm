fn trace(_:u8){}struct D(u8);impl Drop for D{fn drop(&mut self){trace(self.0);}}fn consume(_:D,_:D){}fn f(){consume(D(1),return);}fn main(){f();trace(9);}
