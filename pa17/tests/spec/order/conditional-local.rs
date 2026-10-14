fn trace(_:u8){}struct D(u8);impl Drop for D{fn drop(&mut self){trace(self.0);}}fn f(flag:bool){let d;if flag{d=D(1);}trace(9);}fn main(){f(false);f(true);}
