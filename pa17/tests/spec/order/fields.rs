fn trace(_:u8){}struct D(u8);impl Drop for D{fn drop(&mut self){trace(self.0);}}struct S{a:D,b:D}fn main(){let s=S{b:D(2),a:D(1)};trace(9);}
