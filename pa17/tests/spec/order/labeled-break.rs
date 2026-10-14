fn trace(_:u8){}struct D(u8);impl Drop for D{fn drop(&mut self){trace(self.0);}}fn main(){let result='outer:loop{let a=D(1);loop{let b=D(2);break 'outer D(3);}};trace(9);}
