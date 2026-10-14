fn trace(_:u8){}struct D(u8);impl Drop for D{fn drop(&mut self){trace(self.0);}}fn main(){let result=loop{let a=D(1);break D(2);};trace(9);}
